#!/bin/bash
# RabbitMQ Initialization Script
# This script declares fanout exchanges and queues for all outbox events

set -e

echo "Waiting for RabbitMQ to be ready..."
sleep 5

# RabbitMQ connection parameters
HOST=${RABBITMQ_HOST:-rabbitmq}
USER=${RABBITMQ_USER:-guest}
PASS=${RABBITMQ_PASS:-guest}

# Array of exchange names from routing.yaml
EXCHANGES=(
  "create.server"
  "delete.server"
  "create.channel"
  "delete.channel"
  "update.channel"
  "user.join.server"
  "user.leave.server"
  "role.upsert"
  "role.delete"
  "member.assign.role"
  "member.unassign.role"
  "permission_override.upsert_permission_override"
  "permission_override.delete_permission_override"
)

echo "Creating exchanges and queues..."

# Create each exchange and its corresponding queue
for EXCHANGE in "${EXCHANGES[@]}"; do
  echo "  Creating exchange: $EXCHANGE"
  rabbitmqadmin --host=$HOST --username=$USER --password=$PASS \
    declare exchange name=$EXCHANGE type=fanout durable=true

  # Create a queue with the same name as the exchange
  echo "  Creating queue: ${EXCHANGE}.queue"
  rabbitmqadmin --host=$HOST --username=$USER --password=$PASS \
    declare queue name="${EXCHANGE}.queue" durable=true

  # Bind the queue to the exchange
  echo "  Binding queue ${EXCHANGE}.queue to exchange $EXCHANGE"
  rabbitmqadmin --host=$HOST --username=$USER --password=$PASS \
    declare binding source=$EXCHANGE destination="${EXCHANGE}.queue"
done

echo "All exchanges and queues created successfully!"
