# address bind backend

- [x] verifier
- [x] scanner
- [x] api

## verfiy

```
$ address-bind-be verify -t 0xad5c3e9c15c0da4c8d2ccf65dfc470f3ea84877acd6f6cc3659bd1cd5a0039cf
tx ad5c3e9c15c0da4c8d2ccf65dfc470f3ea84877acd6f6cc3659bd1cd5a0039cf has valid bind info, from: ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah, to: ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah, timestamp: 1757472675162
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
```