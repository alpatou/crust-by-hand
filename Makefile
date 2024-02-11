up:
	docker-compose --env-file .env up -d

down:
	docker-compose --env-file .env down 

build:
	docker-compose --env-file .env build

connect-to-db:
	docker exec -it mysql mysql -p