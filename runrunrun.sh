#!/bin/bash
# 进入指定目录
cd "$(dirname "$0")/clients/cli" || {
    echo "错误：无法进入 clients/cli 目录"
    exit 1
}
INSTANCE_COUNT=2000
BASE_RUN_ID="abuabu"
RUN_MODE=0

echo "Starting $INSTANCE_COUNT instances..."

for i in $(seq 1 $INSTANCE_COUNT); do
    # 构建完整的命令字符串
    CMD="nohup cargo run --release --bin prover -- beta.orchestrator.nexus.xyz --run-id ${BASE_RUN_ID}${i} --run-mode $RUN_MODE > /dev/null 2>&1 &"

    # 打印命令
    echo "Executing: $CMD"

    # 执行命令
    eval "$CMD"

    echo "Started instance $i/$INSTANCE_COUNT"

    # 每启动100个实例暂停2秒
    if [ $((i % 100)) -eq 0 ]; then
        echo "Waiting 2 seconds... Current running instances: $(ps aux | grep '[p]rover' | wc -l)"
        sleep 2
    fi
done

# 显示运行的实例数量
RUNNING_COUNT=$(ps aux | grep '[p]rover' | wc -l)
echo "Finished launching instances"
echo "Actually running instances: $RUNNING_COUNT"