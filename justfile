dev:
    overmind start

api:
    cargo watch -x run

frontend:
    cd frontend && bun dev

build:
  docker build -t registry.lucalise.ca/mc-dashboard .

push:
  docker push registry.lucalise.ca/mc-dashboard

deploy: build push
