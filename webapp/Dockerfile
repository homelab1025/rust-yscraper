# Stage 1: Build the React application
FROM node:24-alpine AS build

WORKDIR /app

# Copy package files and install dependencies
COPY webapp/package*.json ./
RUN npm install

# Copy the rest of the application code
COPY webapp/ ./

# Build the application
RUN npm run build

# Stage 2: Serve the application using Nginx
FROM nginx:stable-alpine

# Copy the built files from the build stage
COPY --from=build /app/dist /usr/share/nginx/html

# Set default environment variable for the backend
ENV BACKEND_URL=http://web-server:3000

# Create a template for nginx configuration
# The nginx image will automatically run envsubst on files in /etc/nginx/templates/*.template
# and output them to /etc/nginx/conf.d/
RUN mkdir -p /etc/nginx/templates
RUN echo 'server { \
    listen 80; \
    location / { \
        root /usr/share/nginx/html; \
        index index.html index.htm; \
        try_files $uri $uri/ /index.html; \
    } \
    location /api/ { \
        proxy_pass ${BACKEND_URL}/; \
        proxy_set_header Host $host; \
        proxy_set_header X-Real-IP $remote_addr; \
    } \
}' > /etc/nginx/templates/default.conf.template

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
