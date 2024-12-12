#!/bin/bash
# 进入指定目录
cd "$(dirname "$0")/clients/cli" || {
    echo "错误：无法进入 clients/cli 目录"
    exit 1
}

INSTANCE_COUNT=1000
BASE_RUN_ID="abuabu"
RUN_MODE=0

echo "Starting $INSTANCE_COUNT instances..."
for i in $(seq 1 $INSTANCE_COUNT); do
    # 构建完整的命令字符串
    CMD="nohup cargo run --release --bin prover -- beta.orchestrator.nexus.xyz --run-id ${BASE_RUN_ID}${i} --run-mode $RUN_MODE &"

    # 打印命令
    echo "Executing: $CMD"

    # 执行命令
    eval "$CMD"

    echo "Started instance $i/$INSTANCE_COUNT"

    # 每个实例后暂停1秒
    sleep 1

    # 每100个实例显示一次运行状态
    if [ $((i % 100)) -eq 0 ]; then
        CURRENT_COUNT=$(ps aux | grep '[p]rover' | wc -l)
        echo "Progress: $i/$INSTANCE_COUNT - Current running instances: $CURRENT_COUNT"
    fi
done

# 显示运行的实例数量
RUNNING_COUNT=$(ps aux | grep '[p]rover' | wc -l)
echo "Finished launching instances"
echo "Actually running instances: $RUNNING_COUNT"