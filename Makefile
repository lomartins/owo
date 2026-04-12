.PHONY: help setup dev db-up db-down db-reset db-logs backend-build backend-test backend-run backend-check web-dev web-build mobile-build clean

help:
	@echo "owo Development Commands"
	@echo ""
	@echo "Setup & Local Development:"
	@echo "  make setup          - Install dependencies and start services"
	@echo "  make dev            - Start all services (Docker + local servers)"
	@echo ""
	@echo "Database:"
	@echo "  make db-up          - Start PostgreSQL"
	@echo "  make db-down        - Stop PostgreSQL"
	@echo "  make db-reset       - Reset database (⚠️ deletes all data)"
	@echo "  make db-logs        - View PostgreSQL logs"
	@echo "  make db-connect     - Connect to PostgreSQL shell"
	@echo ""
	@echo "Backend:"
	@echo "  make backend-build  - Build backend"
	@echo "  make backend-run    - Run backend locally"
	@echo "  make backend-test   - Run backend tests"
	@echo "  make backend-check  - Check code (clippy + fmt)"
	@echo ""
	@echo "Web Frontend:"
	@echo "  make web-dev        - Start web dev server"
	@echo "  make web-build      - Build web for production"
	@echo ""
	@echo "Mobile:"
	@echo "  make mobile-build   - Build mobile app"
	@echo ""
	@echo "Other:"
	@echo "  make clean          - Remove build artifacts"
	@echo "  make logs           - View all Docker logs"

setup:
	@echo "Setting up owo development environment..."
	docker-compose up -d
	cd backend && cargo build
	cd web && npm install
	@echo "✅ Setup complete! Run 'make dev' to start development."

dev: db-up
	@echo "Starting development services..."
	docker-compose up -d
	@echo "📦 Docker services started"
	@echo "🚀 To run backend: cd backend && cargo run"
	@echo "🌐 To run web: cd web && npm run dev"

# Database targets
db-up:
	docker-compose up -d postgres
	@echo "✅ PostgreSQL started on port 5432"

db-down:
	docker-compose down

db-reset:
	@echo "⚠️  Removing all database data..."
	docker-compose down -v
	docker-compose up -d postgres
	@echo "✅ Database reset"

db-logs:
	docker-compose logs -f postgres

db-connect:
	docker-compose exec postgres psql -U owo -d owo_dev

# Backend targets
backend-build:
	cd backend && cargo build --release

backend-run:
	cd backend && cargo run

backend-test:
	cd backend && cargo test

backend-check:
	cd backend && cargo clippy --all-targets && cargo fmt --check

backend-fmt:
	cd backend && cargo fmt

# Web targets
web-dev:
	cd web && npm run dev

web-build:
	cd web && npm run build

# Mobile targets
mobile-build:
	cd mobile && ./gradlew build

# Cleanup
clean:
	cd backend && cargo clean
	cd web && rm -rf node_modules dist build
	docker-compose down

logs:
	docker-compose logs -f

# Docker image building (for CI/CD)
docker-build:
	docker build -t owo-backend:latest .

docker-run: docker-build
	docker run -p 3000:3000 --network owo-network --env-file .env owo-backend:latest
