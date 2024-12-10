#!/bin/bash

# 进入指定目录
cd "$(dirname "$0")/clients/cli" || {
    echo "错误：无法进入 clients/cli 目录"
    exit 1
}

# 询问主名字，默认为 nexus
read -p "请输入主名字 (直接回车默认为 nexus): " main_name
if [ -z "$main_name" ]; then
    main_name="nexus"
fi

# 询问要运行多少个实例
read -p "请输入要运行的实例数量: " instance_count

# 验证输入是否为数字
if ! [[ "$instance_count" =~ ^[0-9]+$ ]]; then
    echo "错误：请输入有效的数字"
    exit 1
fi

# 创建一个新的 tmux 会话
session_name="prover_session"
tmux new-session -d -s "$session_name"

# 为每个实例创建一个新的窗口并运行命令
for ((i=1; i<=instance_count; i++)); do
    # 构建完整的命令
    cmd="cargo run --release --bin prover -- beta.orchestrator.nexus.xyz --run-id ${main_name}${i}"

    if [ $i -eq 1 ]; then
        # 第一个窗口已经存在，只需要发送命令
        tmux send-keys -t "$session_name:0" "cd $(pwd)" C-m
        tmux send-keys -t "$session_name:0" "$cmd" C-m
    else
        # 创建新窗口并发送命令
        tmux new-window -t "$session_name:$((i-1))"
        tmux send-keys -t "$session_name:$((i-1))" "cd $(pwd)" C-m
        tmux send-keys -t "$session_name:$((i-1))" "$cmd" C-m
    fi

    echo "启动实例 $i: $cmd"
    sleep 1
done

# 附加到 tmux 会话
echo "所有实例已启动，正在连接到 tmux 会话..."
sleep 2
tmux attach-session -t "$session_name"