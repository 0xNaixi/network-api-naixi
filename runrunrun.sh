#!/bin/bash

# 设置实例数量
INSTANCE_COUNT=500
# 设置基础运行ID
BASE_RUN_ID="abu"
# 设置运行模式
RUN_MODE=0

echo "Starting $INSTANCE_COUNT instances..."

for i in $(seq 1 $INSTANCE_COUNT); do
    nohup cargo run --release --bin prover -- beta.orchestrator.nexus.xyz --run-id ${BASE_RUN_ID}${i} --run-mode $RUN_MODE > /dev/null 2>&1 &
    echo "Started instance $i/$INSTANCE_COUNT"

    # 每启动100个等待1秒
    if [ $((i % 100)) -eq 0 ]; then
        echo "Waiting 1 second after starting $i instances..."
        sleep 1
    fi
done

echo "All instances started. Use 'ps aux | grep prover' to check running instances"
echo "To kill all instances, use: pkill -f 'prover.*beta.orchestrator'"