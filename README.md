# ✨ Nebulon: A Lightweight Containerization Runtime

Nebulon is a minimalist containerization runtime built in Rust, designed to provide fundamental container management capabilities including process isolation, resource control, and a streamlined command-line interface. It enables users to create, run, and manage isolated environments efficiently, leveraging Linux namespaces and cgroups.

---

## 🚀 Features

*   **Container Lifecycle Management**: Create, start, and stop containers with ease.
*   **Process Isolation**: Utilizes Linux namespaces (Mount, UTS, PID, Network, IPC, User) to isolate container processes from the host system.
*   **Resource Management**: Integrates with cgroups to set memory and CPU usage limits for containers.
*   **Filesystem Management**: Manages container root filesystems, including mounting `/proc` and setting up a dedicated rootfs.
*   **Volume Mounting**: Supports mounting host directories into containers.
*   **CLI Interface**: A user-friendly command-line tool (`nb`) for all container operations.
*   **Image Management**: Basic commands for importing, listing, and removing container images.
*   **Persistent Storage**: Stores container metadata for persistent management.

## 🛠️ Technologies Used

| Technology | Category         | Description                                       |
| :--------- | :--------------- | :------------------------------------------------ |
| **Rust**   | Language         | Primary language for performance and safety.      |
| `clap`     | CLI              | Command-line argument parser for a robust CLI.    |
| `nix`      | System Calls     | Linux syscalls for namespaces, cgroups, and mounts. |
| `anyhow`   | Error Handling   | Flexible error handling.                          |
| `serde`    | Serialization    | For serializing/deserializing container metadata. |
| `log4rs`   | Logging          | Configurable logging system for runtime events.   |
| `uuid`     | Utility          | Generates unique IDs for containers.              |
| `sysinfo`  | System Info      | Gathers system-level information.                 |
| `chrono`   | Date/Time        | For time-based logging and metadata.              |

## 🚦 Getting Started

Follow these steps to get Nebulon up and running on your local machine.

### Installation

To compile and install Nebulon, you'll need a Rust toolchain (Rustup is recommended).

1.  👯‍♀️ **Clone the Repository**:
    ```bash
    git clone https://github.com/Dilibe-Franklin/nebulon.git
    cd nebulon
    ```

2.  🏗️ **Build the Project**:
    ```bash
    cargo build --release
    ```
    This will compile the `nb` executable and place it in `target/release/`.

3.  🔗 **Add to PATH (Optional but Recommended)**:
    For convenience, you can add the compiled executable to your system's PATH:
    ```bash
    sudo cp target/release/nb /usr/local/bin/
    ```
    Alternatively, you can run commands directly using `cargo run --release -- <command>`.

### Environment Variables

Nebulon does not require any specific environment variables for its own operation. However, you can pass environment variables to your containers using the `--env` flag during creation.

## 💡 Usage

Nebulon provides a command-line interface (`nb`) for interacting with containers.

### Basic Commands

*   **Initialize Nebulon**:
    This command sets up the necessary base directories for Nebulon.
    ```bash
    nb init
    ```

*   **Create a Container**:
    Creates a new container with specified configurations. The `rootfs` directory (`/var/lib/nebulon/rootfs` by default) must exist and contain a basic Linux filesystem (e.g., a busybox image).
    ```bash
    nb create \
      --hostname my-container \
      --rootfs /var/lib/nebulon/rootfs \
      --workdir / \
      --env PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
      --uid 1000 --gid 1000 \
      ping google.com
    ```
    _Example `metadata.json` configuration for container ID `nb-317d1158`:_
    ```json
    {
      "id": "nb-317d1158",
      "pid": 0,
      "status": "Created",
      "config": {
        "command": ["ping"],
        "args": ["google"],
        "env_vars": [],
        "working_dirs": "/",
        "hostname": "orca-container",
        "rootfs": "/var/lib/nebulon/rootfs",
        "uid": 1,
        "gid": 1
      }
    }
    ```

*   **Run a Container**:
    Starts a previously created container.
    ```bash
    nb run <container_id>
    ```
    _Example:_
    ```bash
    nb run nb-317d1158
    ```

*   **List Containers**:
    Displays a list of all managed container IDs.
    ```bash
    nb ps
    ```

*   **Stop a Container**:
    Sends a termination signal to a running container.
    ```bash
    nb stop <container_id>
    ```
    _Example:_
    ```bash
    nb stop nb-317d1158
    ```

### Volume Management

*   **Create a Volume**:
    ```bash
    nb volume create myvolume --driver local
    ```

*   **List Volumes**:
    ```bash
    nb volume ls
    ```

*   **Remove a Volume**:
    ```bash
    nb volume rm myvolume
    ```

### Image Management

*   **Import an Image**:
    ```bash
    nb image import /path/to/my_image.tar myapp latest
    ```

*   **List Images**:
    ```bash
    nb image ls
    ```

*   **Remove an Image**:
    ```bash
    nb image rm myapp latest
    ```

## 🤝 Contributing

We welcome contributions to Nebulon! To contribute, please follow these guidelines:

*   🐛 **Report Bugs**: If you find a bug, please open an issue describing the problem and steps to reproduce it.
*   💡 **Suggest Features**: Have an idea for a new feature? Open an issue to discuss it.
*   👨‍💻 **Submit Pull Requests**:
    *   Fork the repository and create a new branch for your feature or bug fix.
    *   Ensure your code adheres to Rust's best practices and the existing coding style.
    *   Write clear, concise commit messages.
    *   Open a pull request with a detailed description of your changes.

## 📝 License

Nebulon is distributed under the [MIT License](https://opensource.org/licenses/MIT). See the `LICENSE` file for more details.

## ✍️ Author

**Dilibe Fidelugwuowo**
*   Email: franklinfidelugwuowo@gmail.com
*   LinkedIn: [Your LinkedIn Profile](https://linkedin.com/in/your_username)
*   Twitter: [@your_twitter_handle](https://twitter.com/your_twitter_handle)

---
[![Rust](https://img.shields.io/badge/Language-Rust-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/Ddilibe/nebulon/actions)
[![Readme was generated by Dokugen](https://img.shields.io/badge/Readme%20was%20generated%20by-Dokugen-brightgreen)](https://www.npmjs.com/package/dokugen)