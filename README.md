# rust_remote_client


## 说明
这是一个远程客户端，本质上其实是通过私有协议栈（proto.rs）, 在一条tcp长链接上模拟多个tcp端口映射

## 这个程序他分2个运行模式：控制端，被控制端
1. 通过去telnet 10.35.146.7:22  如果能通，那么启动模式为控制端，否则就是被控制端
2. 不管是哪个端，先要根据当前时刻的 月，日，时，分，秒，纳秒=》计算出自己的id（10字节）

### 控制端
1.界面上配置，远端局域网中某个ip+端口，调用后端接口，后端会再返回本地的一个端口，给界面显示
xxx.xxx.xxx.xxx:xxx -> 本地端口xxx
2.界面上除了添加映射，还可以删除
3.本地tcp端口，收到的conn，就是一个session
4.如果本地端口的tcp断开了，就发送finmessage消息

### 被控制端
1.收到datamesssage，就会转发数据到指定的target
2.datamesssage中 控制端id+sessionid = 唯一值，如果他不存在，我们就dial targettcpaddr
3.如果targettcpaddr断开了，我们就发送finmessage


## 备注
1. proto.rs是从服务器拷贝过来的，他那边维护，我们不要做修改