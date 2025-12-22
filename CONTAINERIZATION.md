# Containerization Documentation

## Overview
The Smart Loan Recovery System is containerized using Docker for easy deployment and portability. The containerization setup uses a multi-stage build process to create a lightweight production image.

## Prerequisites
- Docker installed on your system
- Basic knowledge of Docker commands

## Dockerfile Explanation

The project uses a multi-stage Docker build:

```dockerfile
FROM rust:1.92 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/smart-loan-recovery /usr/local/bin/
EXPOSE 3000
ENV SERVER_HOST=0.0.0.0
CMD ["smart-loan-recovery"]
```

### Build Stage
- Uses the official Rust 1.92 image as the build environment
- Copies the source code into the container
- Builds the application in release mode for optimal performance

### Runtime Stage
- Uses Debian Bookworm Slim as the base image for a minimal footprint
- Copies only the compiled binary from the build stage
- Sets SERVER_HOST=0.0.0.0 to bind to all network interfaces
- Exposes port 3000 for the web API
- Sets the binary as the default command

## Building the Docker Image

To build the Docker image, run the following command from the project root directory:

```bash
docker build -t smart-loan-recovery .
```

This command:
- Builds the image with the tag `smart-loan-recovery`
- Uses the Dockerfile in the current directory
- May take several minutes on first build due to Rust compilation

## Running the Container

### Basic Usage
Run the application in a container:

```bash
docker run --rm smart-loan-recovery
```

This runs the demo mode and exits.

### Interactive CLI Usage
For interactive use with command-line arguments:

```bash
docker run --rm smart-loan-recovery --help
docker run --rm smart-loan-recovery register-user --name "John Doe" --role borrower
```

### Data Persistence
The application creates SQLite database files (`loans.db`) and JSON backups (`users_backup.json`, `loans_backup.json`) in the working directory. To persist data between container runs:

```bash
docker run --rm -v $(pwd)/data:/app/data smart-loan-recovery
```

**Note**: The current Dockerfile doesn't set a working directory for data persistence. For production use, consider modifying the Dockerfile to use `/app` as the working directory and mounting volumes accordingly.

## Container Benefits

- **Portability**: Run the same application across different environments
- **Isolation**: Application runs in its own containerized environment
- **Lightweight**: Multi-stage build results in a minimal runtime image
- **Reproducibility**: Consistent builds across different systems

## Troubleshooting

### Build Issues
- Ensure Docker has sufficient resources allocated
- Check internet connection for downloading base images
- Verify Rust version compatibility (currently set to 1.81)

### Runtime Issues
- Use `--rm` flag to automatically clean up containers after execution
- Check container logs with `docker logs <container-id>` if needed
- Ensure proper volume mounting for data persistence

## Troubleshooting

### Build Failures

#### "failed to solve: process "/bin/sh -c cargo build --release" did not complete successfully: exit code: 101"

**Cause**: Compilation errors during the Rust build process.

**Debugging Steps**:
1. **Check Rust Version Compatibility**:
   ```bash
   # Check your local Rust version
   rustc --version
   
   # Ensure Dockerfile uses compatible version
   # Update FROM rust:X.Y in Dockerfile if needed
   ```

2. **Test Local Build**:
   ```bash
   # Test build locally first
   cargo check
   cargo build --release
   ```

3. **Check for Platform-Specific Issues**:
   ```bash
   # Build with verbose output
   docker build --progress=plain -t smart-loan-recovery .
   
   # Or run build step manually
   docker run --rm -v $(pwd):/app -w /app rust:1.92 cargo build --release
   ```

4. **Common Issues**:
   - **Version Mismatch**: Ensure Dockerfile Rust version matches your development environment
   - **Missing Dependencies**: Check if all Cargo.toml dependencies are available
   - **Platform Differences**: Linux container vs Windows development might have issues

#### "failed to solve: process "/bin/sh -c cargo build --release" did not complete successfully: exit code: 125"

**Cause**: Docker-specific issues, not compilation errors.

**Debugging Steps**:
1. Check Docker daemon status
2. Ensure sufficient disk space
3. Try building with `--no-cache` flag

### Runtime Issues

#### Container Won't Start
```bash
# Check container logs
docker logs <container-name>

# Run with environment variables
docker run -e RUST_LOG=debug smart-loan-recovery
```

#### Port Already in Use
```bash
# Find what's using port 3000
netstat -tulpn | grep :3000

# Use different port
docker run -p 3001:3000 smart-loan-recovery
```

## Future Enhancements
- Add environment variable support for configuration
- Implement health checks for container orchestration
- Add support for Docker Compose for multi-service setups
- Consider using distroless images for even smaller footprints