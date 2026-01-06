#!/bin/bash
# RabbitMQ Initialization Script
# This script declares the fanout exchange for create_server events

set -e

echo "Waiting for RabbitMQ to be ready..."
sleep 5

# Declare fanout exchange for create_server using rabbitmqadmin
rabbitmqadmin --host=${RABBITMQ_HOST:-rabbitmq} \
  --username=${RABBITMQ_USER:-guest} \
  --password=${RABBITMQ_PASS:-guest} \
  declare exchange name=created_server type=fanout durable=true

echo "Fanout exchange 'beep.community.fanout' created successfully"
