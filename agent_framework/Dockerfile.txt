FROM node:20-alpine

WORKDIR /app

# Install dependencies first (better caching)
COPY package*.json ./
RUN npm install

# Copy source code
COPY tsconfig.json ./
COPY src ./src

# Build TypeScript
RUN npm run build

# Set environment variables
ENV NODE_ENV=production
ENV SUI_NETWORK=testnet

# The server uses stdio transport - it reads from stdin and writes to stdout
# Clawdbot will launch this as a background process
CMD ["node", "dist/index.js"]
