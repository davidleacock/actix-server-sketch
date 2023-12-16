# actix-server-sketch

### Experimenting with actix tcp/http server + mDNS for server discovery <br><br><br>

## ControllerServer 
### Actix TCP server will display incoming tcp Sensor data
`cargo run`

This will start up the ControllerServer on localhost:3000 <br>
Sensor data is displayed on the `localhost:8080/display` endpoint <br><br>



## TCPSensor
### Generates random data and sends to TCP server
`NAME="sensor_1" cargo run`

This will start up the sensor and it should automatically discover the ControllerServer and make a TCP connection and begin streaming random data


