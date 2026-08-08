#!/bin/bash

VM_DISK="vm/my_vm_disk.qcow2"
VM_ISO="vm/debian-12.0.0-amd64-netinst.iso"
VM_NAME="learning-vm"
VM_USER="vagrant"
SSH_PORT=2222

# Reused every boot since this is a fixed local port-forward to a VM whose
# host key can change across reinstalls — treat it like localhost dev, not
# a real remote host.
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p ${SSH_PORT}"

NET_OPTS="-netdev user,id=net0,hostfwd=tcp::${SSH_PORT}-:22 -device virtio-net-pci,netdev=net0"

normal_run() {
    qemu-system-x86_64 -cpu host \
        -name "$VM_NAME" \
        -enable-kvm \
        -m 4G \
        -smp 2 \
        -hda "$VM_DISK" \
        $NET_OPTS \
        -vga std
}

headless_run() {
    if pgrep -f "qemu.*${VM_NAME}" > /dev/null; then
        echo "VM already running."
        return
    fi
    qemu-system-x86_64 -cpu host \
        -name "$VM_NAME" \
        -enable-kvm \
        -m 4G \
        -smp 2 \
        -hda "$VM_DISK" \
        $NET_OPTS \
        -display none \
        -daemonize
    echo "VM started headless. SSH: ssh -p ${SSH_PORT} ${VM_USER}@localhost"
}

reinstall_run() {
    qemu-system-x86_64 -cpu host \
        -name "$VM_NAME" \
        -enable-kvm \
        -m 4G \
        -smp 2 \
        -hda "$VM_DISK" \
        -cdrom "$VM_ISO" \
        -boot d \
        $NET_OPTS \
        -vga std
}

ssh_connect() {
    if ! pgrep -f "qemu.*${VM_NAME}" > /dev/null; then
        echo "VM is not running. Start it first (gui or headless)."
        return 1
    fi
    ssh $SSH_OPTS ${VM_USER}@localhost
}

SRC_DIR="learning"
DEST_DIR="~/HTTP_Server/learning"

transfer_files() {
    if ! pgrep -f "qemu.*${VM_NAME}" > /dev/null; then
        echo "VM is not running. Start it first (gui or headless)."
        return 1
    fi
    rsync -avz --exclude 'target' --exclude '.git' \
        -e "ssh $SSH_OPTS" \
        "${SRC_DIR}/" "${VM_USER}@localhost:${DEST_DIR}/"
}

wait_for_ssh() {
    echo "Waiting for the guest to finish booting and sshd to come up..."
    local elapsed=0
    local timeout=180
    while [ $elapsed -lt $timeout ]; do
        if timeout 3 bash -c "</dev/tcp/localhost/${SSH_PORT}" 2>/dev/null; then
            echo "SSH port is open."
            return 0
        fi
        echo -n "."
        sleep 5
        elapsed=$((elapsed + 5))
    done
    echo ""
    echo "Timed out waiting for SSH. Check the guest console (gui) — is it actually booted, is sshd running?"
    return 1
}

setup_keys() {
    if ! pgrep -f "qemu.*${VM_NAME}" > /dev/null; then
        echo "VM is not running. Start it first (gui or headless)."
        return 1
    fi

    # BatchMode probe above will fail on password-only auth (expected pre-key-setup) —
    # that's fine, it just confirms the port is reachable at all before we hang ssh-copy-id.
    wait_for_ssh || true

    local key_path="$HOME/.ssh/id_rsa"
    if [ ! -f "$key_path" ]; then
        echo "No SSH key found, generating one..."
        ssh-keygen -t rsa -b 2048 -f "$key_path" -N ""
    fi

    echo "Copying key to VM — you'll be asked for ${VM_USER}'s password ONE more time..."
    if command -v ssh-copy-id > /dev/null 2>&1; then
        ssh-copy-id $SSH_OPTS -i "${key_path}.pub" ${VM_USER}@localhost
    else
        cat "${key_path}.pub" | ssh $SSH_OPTS ${VM_USER}@localhost \
            "mkdir -p ~/.ssh && chmod 700 ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"
    fi

    if ssh -o BatchMode=yes $SSH_OPTS ${VM_USER}@localhost exit 2>/dev/null; then
        echo "Passwordless SSH is working. ssh and sync won't prompt anymore."
    else
        echo "Something didn't take — try 'ssh $SSH_OPTS ${VM_USER}@localhost' manually to see the actual error."
    fi
}

stop_vm() {
    if ! pgrep -f "qemu.*${VM_NAME}" > /dev/null; then
        echo "VM is not running."
        return
    fi
    pkill -f "qemu.*${VM_NAME}"
    sleep 1
    if pgrep -f "qemu.*${VM_NAME}" > /dev/null; then
        pkill -9 -f "qemu.*${VM_NAME}"
    fi
    echo "VM stopped."
}

usage() {
    cat <<- USAGE
    Usage: $0 <command>

      gui         Start VM with display
      headless    Start VM in background, no display
      reinstall   Boot from install ISO
      keys        One-time SSH key setup (run this first)
      ssh         SSH into the running VM
      sync        rsync ${SRC_DIR}/ to the VM
      stop        Stop the running VM
	USAGE
}

##### Main #####
case "$1" in
    gui) normal_run ;;
    headless) headless_run ;;
    reinstall) reinstall_run ;;
    keys) setup_keys ;;
    ssh) ssh_connect ;;
    sync) transfer_files ;;
    stop) stop_vm ;;
    *) usage ;;
esac