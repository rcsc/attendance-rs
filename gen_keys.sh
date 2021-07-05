#!/bin/bash
openssl ecparam -name prime256v1 -genkey -noout -out private_key.ec.pem
openssl pkcs8 -topk8 -nocrypt -in private_key.ec.pem -out private_key.pem
rm private_key.ec.pem
# Perhaps get rid of -nocrypt if you want the key to have a password, but I don't think that'll work for this program
openssl ec -in private_key.pem -pubout -out public_key.pem
