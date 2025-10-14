# Load variables from .env file
set dotenv-load

# --- Variables ---
# The name for the Docker image.
image_name := "evaluate-app"
# The tag for the Docker image.
image_tag := "latest"

# --- Recipes ---

# Build the Docker image for the application.
build:
    @echo "Building Docker image: {{image_name}}:{{image_tag}}"
    docker build -t {{image_name}}:{{image_tag}} .

# Run the application inside a Docker container.
# Requires a .env file with DATABASE_URL.
run: build
    @echo "Running container from image {{image_name}}:{{image_tag}}"
    docker run --rm -it -p 8080:8080 --env-file .env {{image_name}}:{{image_tag}}

# Stop and remove any running containers with the same image name.
stop:
    @echo "Stopping and removing any running '{{image_name}}' containers..."
    @docker ps -q --filter ancestor={{image_name}} | xargs -r docker stop | xargs -r docker rm

# Default recipe
default: build