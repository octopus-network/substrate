duration=2h

# start mock server
pids=()
cd mock_server
go run mock_json_server.go & 
pids+=($!)

# start node
cd ../scripts
pwd
timeout $duration bash start-alice.sh & 
pids+=($!)
timeout $duration bash start-bob.sh & 
pids+=($!)
timeout $duration bash start-charlie.sh & 
pids+=($!)
timeout $duration bash start-dave.sh & 
pids+=($!)
timeout $duration bash start-eve.sh & 
pids+=($!)

# start js_client
cd ../js_client
node appchain.js &
pids+=($!)

wait "${pids[@]}"
exit


