API_HOST="127.0.0.1:8080"

curl -X POST -H "Content-Type: application/json" -d @city.json http://${API_HOST}/api/data/city
curl -X POST -H "Content-Type: application/json" -d @station.json http:/${API_HOST}/api/data/station
curl -X POST -H "Content-Type: application/json" -d @train_type.json http://${API_HOST}/api/data/train_type
curl -X POST -H "Content-Type: application/json" -d @train_number.json http://${API_HOST}/api/data/train_number
curl -X POST -H "Content-Type: application/json" -d @hotels.json http://${API_HOST}/api/data/hotel
7z x dish_takeaway.7z
curl -X POST -H "Content-Type: application/json" -d @dish_takeaway.json http://${API_HOST}/api/data/dish_takeaway
rm dish_takeaway.json