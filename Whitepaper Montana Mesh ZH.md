# Montana：城市自治网状网络

**Alejandro Montana**
[github.com/efir369999/Montana](https://github.com/efir369999/Montana) · [montana.quest](https://montana.quest)

---

## 摘要

要在没有可信中介、不依赖经典密码学的前提下,在各方之间传递价值,网络必须同时解决三个问题:事件全局排序的一致性、对被动观察者和主动审查者的传输层保护、以及在个别节点失效或被夺取时的基础设施韧性。现有系统只解决其中一个。Bitcoin 提供共识,但不提供传输隐私。Tor 提供传输隐私,但不提供共识。Tailscale 与 WireGuard 提供 peer-to-peer 互联,但不提供基础设施自治。Montana 试图在一个系统中同时具备这三种属性。它由两个相互耦合的层组成:**TimeChain**——一条后量子区块链,其稀缺资源是时间(而非区块空间或手续费),以及 **Mesh VPN**——一个城市节点的联邦,每个节点开放其所在区域的互联网,并在邻近节点失效时承担其客户端。不变的隐喻:每个节点都是地图上的一座城市;由 VPN 城市组成的网络就是互联网。

---

## 1. 引言

Bitcoin [1] 证明了去中心化的货币共识无须可信中介即可实现。Tor [11] 证明了在公共网络中匿名路由流量是可行的。WireGuard [12] 及其衍生方案表明,基于现代密码学构建简单且高速的 peer-to-peer VPN 是可能的。这些系统都不能同时具备 Montana 所追求的三项属性——面向 ≥10⁹ 用户规模的自治互联网所需的:全局排序、传输隐私、以及自治基础设施。

此外,Bitcoin 及其后继者还存在两个未解决的脆弱性。第一,所有生产级区块链都将签名安全建立在椭圆曲线离散对数假设之上。Shor 算法 [8] 在足够大的量子计算机上以多项式时间打破这些假设。NIST 在 2024 年标准化了后量子签名与密钥封装机制(FIPS 203 [2]、204 [3]、205);主要链尚未迁移。第二,基于手续费的反垃圾机制在大规模采用下扩展不佳:拥堵时小额操作被价格挤出,空闲时垃圾以边际成本回归。Layer-2 系统(状态通道、rollup)只是转移经济成本,并未消除底层稀缺。

Montana 提出:一条签名安全完全建立在后量子原语之上、反垃圾机制基于时间(而非金钱)的链 [13]。链通过 SHA-256 上的可验证延迟函数(VDF)[5,6,7] 推进,产生约 60 秒一格的全局有序窗口。窗口内的操作受三种独立的、由时间派生的稀缺资源限制:每身份每窗口、账户链长度、资历。在此层之上,协议部署第二层——网状 VPN,使用 Reality(xray)[14] 将流量伪装为对合法公开目标的常规 TLS 握手。

---

## 2. 架构:两层、一网

Montana 由同一组节点物理实现的两个层组成:

**第 1 层——TimeChain。** 全局时钟 + 身份注册 + 状态。负责:事件排序、节点运营商注册、发行核算、状态保存。详细论述参见 [Whitepaper Montana ZH](Whitepaper%20Montana%20ZH.md);本文第 4 节为摘要。

**第 2 层——网状 VPN。** 通过城市节点路由用户流量。负责:传输隐私、规避审查、节点间故障转移。第 5–7 节展开。

两层并非独立:愿意全程参与的运营商必须同时运行 TimeChain 节点(`montana-node`)和 Reality 配置的 VPN 服务器(`xray`)。TimeChain 提供自治身份和在线证明;VPN 为用户提供带宽。无 TimeChain,VPN 节点只是普通 VPS;无 VPN,TimeChain 节点只是没有有用负载的观察者。两者结合方为自治网络节点。

---

## 3. 城市隐喻

Montana 节点 = 地图上的一座城市。截至 2026-05-10,网络由三座城市组成:

- **莫斯科**(55.7558° N, 37.6173° E)—— TimeChain Active 验证者,窗口发行者;
- **法兰克福**(50.1109° N, 8.6821° E)—— TimeChain candidate,VPN origin;
- **赫尔辛基**(60.1699° N, 24.9384° E)—— TimeChain candidate,赫尔辛基为法兰克福做 VPN front。

隐喻不是装饰性的。它强制三个实现不变量:

(a)**每座城市开放其所在区域。** 选择"赫尔辛基"的用户以赫尔辛基的视角到达公共互联网:从赫尔辛基的 ASN、用赫尔辛基的 DNS 解析、具有赫尔辛基对其他地区被屏蔽资源的可达性。

(b)**城市相互承担。** 当节点失效或被夺取时,余下的城市通过 6.4 节描述的联邦机制接收其客户端。不存在中央故障转移点。

(c)**城市的网络就是互联网。** 终端用户不再区分"使用互联网"与"使用 Montana"——对于加入者,Montana 即互联网。这是终极目标;第 9 节描述当前的部分实现状态。

---

## 4. 第 1 层——TimeChain(摘要)

完整描述见 `Montana Protocol v35.25.0` 与 TimeChain 白皮书 [13]。此处仅给出与第 2 层相关的部分。

**窗口。** 设 `T_r` 为窗口 `r` 的 VDF 输出。链按 `T_r = SHA-256^D (T_{r-1})` 推进,其中 `T_0` 为创世种子,`D` 为每窗口迭代数。在当前 epoch,`D = 325 000 000`,将窗口校准至 commodity x86_64 上约 60 秒。`D` 每 `τ₂ = 20 160` 窗口(约 14 天)按典范公式重新校准。

**发行。** 每个窗口准确铸造 `13 Ɉ = 13·10⁹ nɈ`。供应由闭式公式给出:`supply(W) = 13·(W+1) Ɉ`。撰文时 `W = 34 922`,supply ≈ 454 000 Ɉ。

**节点注册、队列与 welcome 奖励。** 新运营商必须构建长度为 `τ₂` 窗口的 candidate VDF 链(M-class Mac 上约 10 小时实际墙钟)。这是 Sybil 防御:N 个身份需要 N 条 candidate 链。完成 candidate VDF 后,节点提交 `NodeRegistration` 并进入 `CandidatePool` 队列。准入**严格每窗口一个**:在 admission 窗口,被纳入者获得该窗口全部发行(13 Ɉ)作为 welcome 奖励(「一个时间窗口 = 进入 Montana 的第一步」);Active 验证者在该窗口放弃自己的发行。闭式 `supply(W) = 13·(W+1) Ɉ` 不变。若队列中有 N 个就绪候选,全部准入需要 N 个窗口(约 N 分钟)。

**当前网络状态**(2026-05-10 实时快照):

| 城市 | Phase | window | NodeTable | 余额 |
|---|---|---|---|---|
| 莫斯科 | Active | 34922 | 1 | 453 388 Ɉ |
| 法兰克福 | CandidateVdf 42% | 34920 | 1 | 0 |
| 赫尔辛基 | CandidateVdf 4% | 34901 | 1 | 0 |
| Mac(候选) | CandidateVdf 0.5% | 100 | 0 | 0 |

当前仅有一个 Active 验证者——莫斯科。法兰克福和赫尔辛基在数小时墙钟内完成 candidate VDF 并注册;之后 NodeTable 增至三。

---

## 5. 第 2 层——网状 VPN

### 5.1. 传输:xray Reality

每个 VPN 节点运行 [xray](https://github.com/XTLS/Xray-core),inbound 协议为 VLESS 之上的 Reality [14]。Reality 是 TLS 1.3 的修改版,客户端向真实公开目标(如 `www.googletagmanager.com`)发起握手,但首响应来自代理服务器;对 DPI 观察者,整个握手与对该公开站点的常规 TLS 不可区分。`xtls-rprx-vision` 流进一步降低 content-stream 的特征。

节点上单 inbound 的基线配置:

```json
{
  "port": 443,
  "protocol": "vless",
  "streamSettings": {
    "network": "tcp",
    "security": "reality",
    "realitySettings": {
      "dest": "www.googletagmanager.com:443",
      "serverNames": ["www.googletagmanager.com"],
      "shortIds": ["<每节点 8 hex bytes>"],
      "privateKey": "<X25519 私钥——每节点>"
    }
  },
  "settings": {
    "clients": [{ "id": "<UUID>", "flow": "xtls-rprx-vision" }],
    "decryption": "none"
  }
}
```

每个节点持有**自己的** keypair 与**自己的** UUID 客户端列表。节点间协调仅在联邦层级(6.3 节),不在 keymaterial 层级。

### 5.2. 客户端

终端用户安装兼容客户端(iOS 用 Happ,Android 用基于 v2rayNG 的 `Монтана.apk`,桌面端用 Hiddify 或 v2box),并订阅单一 sub URL `https://montana.quest/vpn/sub`。sub 提供网络中所有节点 `vless://` URL 的 base64 串联,每 5 分钟刷新一次(6.3 节)。客户端在某节点失效时自动切换。

### 5.3. 单城市 sub

除联邦池外,每座活跃 VPN 城市还拥有自己的 endpoint:

- `GET /vpn/city/fra`——法兰克福的 vless URL;
- `GET /vpn/city/fin`——赫尔辛基的 vless URL;
- `GET /vpn/city/msk`——目前 404(莫斯科当前为 `node only` 模式,见第 9 节)。

单城市 sub 是用户显式选择城市的界面入口(例如 `montana.quest/net` 上的城市地图)。

---

## 6. 城市间联邦

### 6.1. 原则:本地真实,全局聚合

每个节点只知道关于自己的真相。"城市 X 的 VPN 配置是什么"这一查询的答案是**节点 X 自己发布的内容**。没有中心化数据库;聚合器仅从节点收集真相。

### 6.2. 节点真相之源:`my-vpn.json`

每个节点本地发布 `/var/lib/montana-net/my-vpn.json`:

```json
{
  "node": "frankfurt",
  "primary": false,
  "vless": "vless://...@<host>:443?...#Montana%20FRA"
}
```

该文件仅可由可信的聚合器节点(莫斯科)通过 SSH 访问,需提供专用公钥 `vpn_stats2`,被限制为 `forced-command cat /var/lib/montana-net/my-vpn.json`。不存在公开 HTTP 暴露。

### 6.3. 聚合器:`montana-sub.timer`

莫斯科运行 systemd 计时器 `montana-sub.timer`(每 5 分钟),调用 `/opt/montana-net/sub-aggregator.sh`。脚本:

1. 通过 SSH(forced-command)从每个已知节点拉取 `my-vpn.json`。
2. 收集 `vless://` 列表,排序(primary 优先)。
3. 用 `\n` 串联,base64 编码。
4. 写入 `/var/www/montana_quest/vpn/sub`。

伴生收集器 `/opt/montana-net/aggregator.sh` 从相同节点收集 `peers.json`,发布 `/var/www/montana_quest/vpn/network.json`——联邦健康视图。

### 6.4. 故障转移图

当前 fronting 拓扑:

- **赫尔辛基为法兰克福做 front。** 联邦池中赫尔辛基的 vless URL 指向 `cdn.montana.quest:443`,该地址被代理至赫尔辛基作为主入口。若赫尔辛基失效,客户端通过 `<exit-de>:443`(同一 sub 中的次级 URL)直接连接法兰克福。这一关系在 `cities.json` 中通过 `vpn.fronts` 与 `vpn.fronted_by` 字段表达。
- **莫斯科尚非 VPN。** 当莫斯科启动 VPN(路线图,第 9 节)时,作为第三入口加入池,与法兰克福和赫尔辛基对等。

### 6.5. 健康探测

`peer-health.py`(莫斯科,同一计时器)以目标 SNI 对每个 VPN endpoint 执行 TLS 握手,结果写入 `/var/www/montana_quest/vpn/health.json`。聚合器在临时失效时不将节点逐出 sub——客户端自行切换。健康数据用于 explorer 显示(`montana.quest/net`)。

---

## 7. 默认隐私

### 7.1. TimeChain 层

Account ID 为 `SHA-256(public_key)`。链本身不要求 KYC 元数据。余额是公开的(同 Bitcoin),但昵称及账户间联系默认不暴露——用户自主选择透露内容。详见单独文档"Privacy by default"。

### 7.2. VPN 层

Reality 将握手伪装为对合法公开目标的常规 TLS。观察握手的 DPI 无法区分 Montana-VPN 与对 `www.googletagmanager.com` 的访问。SNI 与证书均匹配公开目标。负载加密为 TLS 1.3(于 Reality 之上)叠加 VLESS 封装;密钥逐会话协商。

### 7.3. Explorer 层

公开仪表板(`montana.quest/net`)**不在 JSON 与 HTML 中暴露节点 IP**。节点坐标精度为城市级别(莫斯科、法兰克福、赫尔辛基),不到数据中心层级。不发布托管商名称。这降低了针对性 DDoS 与社会工程攻击的表面。

---

## 8. 规模

基线目标是支撑 ≥10⁹ 活跃用户。Montana 的每一项架构决策都对照此基线检验;不可扩展的机制无须讨论即被舍弃。详见单独文档"Scale baseline 1B+"。

各层估算:

**TimeChain。** AccountTable 随每次新注册而增长。在 10⁹ 账户、平均记录 ≈2 KB 的情况下,表的量级约为 2 TB。无法置于 RAM,但适合单节点 SSD,前提是节点仅维护活跃集索引。13 Ɉ/窗口 × 525 600 窗口/年 ≈ 6.83M Ɉ/年——相对于宣称的 supply 是可接受的通胀。

**网状 VPN。** 每用户典型 5 Mbps 负载下,具 10 Gbps 上行的节点服务约 2 000 并发活跃会话。10⁹ 用户在 1% 并发活跃假设下约为 10⁷ 活跃流,需要约 5 000 节点。这是联邦的目标:数千座城市的网络。当前的三座是起点。

---

## 9. 当前状态与路线图

### 9.1. 截至 2026-05-10

- **TimeChain**:3 个节点(莫斯科 Active,法兰克福+赫尔辛基处于 candidate-VDF 中),1 个候选(Mac)。Genesis 为 2026-01-09。Window ≈ 35 000。
- **VPN**:3 个活跃点(莫斯科 :2053、法兰克福 :443、赫尔辛基 :443)。赫尔辛基为法兰克福做 front;莫斯科作为独立的第三个 origin。联邦 `/vpn/sub` 聚合三者。
- **Explorer**:`montana.quest/net`——4 节点的实时仪表板,移动端适配,不暴露 IP。
- **城市地图**:后端 `/net/cities.json` 已就绪,`/vpn/city/{msk,fra,fin}` 提供单城市 URL。三座城市均标记为 VPN 节点。可视地图为下一步。

### 9.2. 近期

- 法兰克福和赫尔辛基完成 candidate-VDF 并注册为 Active 验证者。节点间 AccountTable / supply 的偏差归零。
- `montana.quest/net` 上的可视城市地图——独立的前端迭代。

### 9.3. 中期

- 联邦扩张:按需新增城市节点。Onboarding——启动 `montana-node` + xray Reality + 发布 `my-vpn.json`。聚合器自动接入。
- 移动分发:`Монтана.apk` 已构建(基于 v2rayNG 重打包,keystore = genesis 秘密)。iOS 对应是通过 `/vpn/sub` 的 Happ deeplink。

### 9.4. 远期

- 数十至数百座城市。联邦 sub-pool 按区域分片。健康探测成为 consensus 的一部分(节点超过 7 天未响应将通过专用操作从 `NodeTable` 中剔除)。
- 每座城市——自有的 ML-DSA-65 身份、自有的运营商账户、自有的 VPN keypair。

---

## 10. 参考文献

[1] Nakamoto S. *Bitcoin: A Peer-to-Peer Electronic Cash System*. 2008.
[2] NIST FIPS 203. *Module-Lattice-Based Key-Encapsulation Mechanism Standard*. 2024.
[3] NIST FIPS 204. *Module-Lattice-Based Digital Signature Standard*. 2024.
[4] NIST FIPS 180-4. *Secure Hash Standard*. 2015.
[5] Boneh D., Bonneau J., Bünz B., Fisch B. *Verifiable Delay Functions*. CRYPTO 2018.
[6] Wesolowski B. *Efficient Verifiable Delay Functions*. EUROCRYPT 2019.
[7] Pietrzak K. *Simple Verifiable Delay Functions*. ITCS 2019.
[8] Shor P. *Polynomial-Time Algorithms for Prime Factorization and Discrete Logarithms on a Quantum Computer*. SIAM J. Comput., 1997.
[9] Grover L. *A Fast Quantum Mechanical Algorithm for Database Search*. STOC 1996.
[10] *Montana Protocol v35.25.0*. Montana spec, 2026.
[11] Dingledine R., Mathewson N., Syverson P. *Tor: The Second-Generation Onion Router*. USENIX Security 2004.
[12] Donenfeld J. *WireGuard: Next Generation Kernel Network Tunnel*. NDSS 2017.
[13] *Whitepaper Montana ZH*(TimeChain layer)—— `Montana/Montana-Protocol/Whitepaper Montana ZH.md`。
[14] *XTLS Reality*—— `github.com/XTLS/Xray-core/discussions/1295`。

---

*本文档以三种语言发布:俄语(`Whitepaper Montana Mesh RU.md`)、英语(`Whitepaper Montana Mesh.md`)、中文(本文)。三者内容相同;若有出入,典范版本为俄语版。*
