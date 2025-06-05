curl -X POST -H "Content-Type: application/json" -d @city.json http://127.0.0.1:8080/api/data/city
curl -X POST -H "Content-Type: application/json" -d @station.json http://127.0.0.1:8080/api/data/station
curl -X POST -H "Content-Type: application/json" -d @train_type.json http://127.0.0.1:8080/api/data/train_type
curl -X POST -H "Content-Type: application/json" -d @train_number.json http://127.0.0.1:8080/api/data/train_number
curl -X POST -H "Content-Type: application/json" -d @hotels.json http://127.0.0.1:8080/api/data/hotel
7z x dish_takeaway.7z
curl -X POST -H "Content-Type: application/json" -d @dish_takeaway.json http://127.0.0.1:8080/api/data/dish_takeaway
rm dish_takeaway.json