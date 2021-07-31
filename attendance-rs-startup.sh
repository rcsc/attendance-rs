#!/bin/sh

echo STARTING SERVER
/sqlx migrate run
/attendance-rs
