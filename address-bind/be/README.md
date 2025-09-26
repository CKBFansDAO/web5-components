# address bind backend

- [x] verifier
- [x] scanner
- [x] api

## verfiy

```
$ address-bind-be verify -t 0xad5c3e9c15c0da4c8d2ccf65dfc470f3ea84877acd6f6cc3659bd1cd5a0039cf
tx ad5c3e9c15c0da4c8d2ccf65dfc470f3ea84877acd6f6cc3659bd1cd5a0039cf has valid bind info, from: ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah, to: ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah, timestamp: 1757472675162
```

## indexer

```
$ address-bind-be indexer -s 18467780
current_height: 18467780
current_height: 18467781
from: ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah, to: ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah, timestamp: 1757472675162
current_height: 18467782
current_height: 18467783
current_height: 18467784
```

## api

```
API documentation including:

1.
   /by_from/:from endpoint
   
   - Description: Query binding information by from address
   - Parameters: from - The source address to query
   - Response: Array containing target address (to) and timestamp for each binding record
2.
   /by_to/:to endpoint
   
   - Description: Query binding information by to address
   - Parameters: to - The target address to query
   - Response: Array containing source address (from) and timestamp for each binding record
   - Note: For each from address, only returns the record with the latest timestamp
3.
   /health endpoint
   
   - Description: Check the health status of the API server
   - Response: JSON object with a single property "status" set to "OK" if the server is running
```

```
$ curl http://localhost:9533/by_from/ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ahah
[{"timestamp":1757472675162,"to":"ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah"}]

$ curl -vv http://localhost:9533/by_to/ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah

[{"from":"ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah","timestamp":1757472675162}]
```
