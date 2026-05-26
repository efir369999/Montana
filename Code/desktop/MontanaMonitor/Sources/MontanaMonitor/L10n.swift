import SwiftUI
import Foundation

enum AppLang: String, CaseIterable, Identifiable {
    case ru, en, zh
    var id: String { rawValue }
    var label: String {
        switch self {
        case .ru: return "РУС"; case .en: return "EN"; case .zh: return "中文"
        }
    }
}

@MainActor
final class AppLocale: ObservableObject {
    @Published var lang: AppLang {
        didSet { UserDefaults.standard.set(lang.rawValue, forKey: "MontanaLang") }
    }
    init() {
        if let raw = UserDefaults.standard.string(forKey: "MontanaLang"),
           let l = AppLang(rawValue: raw) {
            self.lang = l
        } else {
            let sys = Foundation.Locale.current.language.languageCode?.identifier ?? "ru"
            switch sys {
            case "ru": self.lang = .ru
            case "zh": self.lang = .zh
            default:   self.lang = .en
            }
        }
    }
    func t(_ key: String) -> String {
        return Self.dict[lang]?[key] ?? Self.dict[.ru]?[key] ?? key
    }
    static let dict: [AppLang: [String: String]] = [.ru: ru, .en: en, .zh: zh]

    // MARK: - RU
    static let ru: [String: String] = [
        // Tabs / sidebar
        "tab.wallet": "Кошелёк", "tab.vpn": "ВПН", "tab.network": "Сеть",
        "side.status": "статус",
        "side.node.stopped": "узел: остановлен",
        "side.node.running": "узел: %@",
        "side.net.window": "сеть: окно %@",
        "side.vpn.on": "ВПН: %@",
        "side.version": "Montana Ядро 0.1",

        // Onboarding
        "onboard.title": "Добро пожаловать в Монтану",
        "onboard.subtitle": "Это полный узел сети Монтана. Установка приложения = ваш ноутбук становится частью мейннета.",
        "onboard.step1.t": "Создаём ключ",
        "onboard.step1.b": "24 слова — единственный способ восстановить кошелёк. Запишите на бумаге, никому не показывайте.",
        "onboard.step2.t": "Запускается узел",
        "onboard.step2.b": "Локальный процесс montana-node стартует в фоне. Подключается к узлам сети и качает текущее состояние.",
        "onboard.step3.t": "Узел считает VDF",
        "onboard.step3.b": "Постквантовый «таймер» доказывает что вы провели в сети τ₂ = 20 160 окон. Можно закрыть ноутбук — при следующем запуске продолжит с того же окна.",
        "onboard.step4.t": "Регистрируетесь как валидатор",
        "onboard.step4.b": "После τ₂ = 20 160 окон узел подаёт заявку. В ближайшее окно selection (каждые 336 окон) вас принимают.",
        "onboard.step5.t": "Получаете эмиссию",
        "onboard.step5.b": "Active валидатор получает 13 Ɉ за каждое окно пока узел работает.",
        "onboard.lifespan.t": "Узел живёт пока приложение Монтаны запущено",
        "onboard.lifespan.b": "Свернуть → работает. Закрыть → останавливается. При следующем открытии продолжает с того же окна VDF.",
        "onboard.btn.create": "Создать кошелёк и запустить узел",
        "onboard.recover": "Уже есть 24 слова — восстановить",
        "onboard.words": "%d из 24",
        "onboard.btn.restore": "Восстановить",

        // Mnemonic capture
        "mnemonic.warn": "Запишите 24 слова. Без них вы потеряете доступ к кошельку. После «Сохранил» их больше не показать.",
        "mnemonic.copy": "Скопировать",
        "mnemonic.saved": "Сохранил — запустить узел",
        "mnemonic.copied": "скопировано",

        // Wallet
        "wallet.title": "Кошелёк",
        "wallet.subtitle": "Узел Монтаны (локальный)",
        "wallet.balance": "баланс",
        "wallet.crypto.label": "криптосистема",
        "wallet.crypto.val": "ML-DSA-65",
        "wallet.account_id": "ваш account_id",
        "wallet.send": "Отправить",
        "wallet.receive": "Получить",
        "wallet.wipe": "Удалить кошелёк и узел",
        "wallet.node_stopped": "Узел остановлен",
        "wallet.node_off_note": "Чтобы получать обновления баланса и эмиссию — узел должен работать. Первый запуск: дождитесь τ₂ = 20 160 окон.",
        "wallet.start_node": "Запустить узел",
        "wallet.candidate": "Прогресс кандидата",
        "wallet.candidate_note": "При достижении τ₂ = 20 160 окон узел подаст заявку на регистрацию.",
        "wallet.candidate_of": "%@ из %@ окон",

        // Wipe alert
        "wipe.title": "Удалить кошелёк?",
        "wipe.info": "Это удалит identity локального узла. Без 24 слов восстановить невозможно.",
        "wipe.ok": "Удалить",
        "wipe.cancel": "Отмена",

        // Network tab
        "net.title": "Сеть Монтана",
        "net.subtitle": "Мейннет",
        "net.synced": "синхронизирован с сетью",
        "net.summary": "Активных узлов %d из %d; внешних операторов: %d.",
        "net.no_link": "нет связи с сетью",
        "net.my_node": "Ваш узел в сети",
        "net.synced_to": "синхронизировано: %@ из %@",
        "net.not_started": "Узел не запущен. Перейдите во вкладку Кошелёк, чтобы создать или запустить.",
        "net.connecting": "подключение…",
        "net.supply": "supply",
        "net.window": "окно",
        "net.nodes": "узлов",
        "net.active_total": "активных/всего",
        "net.explorer": "Эксплорер",
        "net.last_update": "обновлено %@",
        "net.install_node": "Любой может развернуть узел Монтаны командой на чистом Linux VPS:",

        // Node phases
        "phase.unknown":      "не запущен",
        "phase.bootstrap":    "подключение к сети",
        "phase.candidateVdf": "кандидат · VDF",
        "phase.registered":   "ожидание окна включения",
        "phase.active":       "активный валидатор",

        // VPN
        "vpn.title": "ВПН Монтана",
        "vpn.state.off": "отключено",
        "vpn.state.conn": "подключение…",
        "vpn.state.on": "подключено",
        "vpn.state.err": "ошибка",
        "vpn.servers": "Серверы",
        "vpn.connect": "Подключить",
        "vpn.disconnect": "Отключить",
        "vpn.loading": "загрузка серверов…",
        "vpn.connecting_to": "запускаем xray → %@…",
        "vpn.connected_to": "подключено к %@ %@",
        "vpn.disconnecting": "отключаем…",
        "vpn.error": "ошибка",
        "vpn.error_label": "ошибка: %@",

        // App menu (Help)
        "menu.site": "Сайт Montana",
        "menu.explorer": "Эксплорер",        "vpn.port_busy": "Порт 10808 занят другим ВПН-клиентом (например Happ). Закройте его, чтобы Montana управляла соединением.",
        "vpn.first_setup.title": "Montana — однократная настройка",
        "vpn.first_setup.body": "Чтобы направлять трафик через узлы Монтаны, нужно один раз разрешить управление системным прокси. macOS сейчас попросит пароль администратора. После этого ВПН включается и выключается мгновенно, без запросов.",
        "vpn.first_setup.ok": "Разрешить",
        "vpn.first_setup.cancel": "Отмена",

        // VPN extended
        "vpn.subs":               "Подписки",
        "vpn.sub.add":            "Добавить подписку",
        "vpn.sub.url_ph":         "URL подписки или vless://",
        "vpn.sub.refresh":        "Обновить",
        "vpn.sub.refresh_all":    "Обновить все",
        "vpn.sub.delete":         "Удалить",
        "vpn.sub.last_update":    "обновлено: %@",
        "vpn.sub.never":          "никогда",
        "vpn.sub.servers_count":  "%d серверов",
        "vpn.sub.auto":           "Авто-обновление каждые %d мин",
        "vpn.ping_all":           "Пинговать все",
        "vpn.add_server":         "Добавить сервер",
        "vpn.server.name_ph":     "Название (необязательно)",
        "vpn.server.url_ph":      "vless://… (вставьте URL)",
        "vpn.server.add":         "Добавить",
        "vpn.server.cancel":      "Отмена",
        "vpn.empty":              "Серверов нет — добавьте подписку или вставьте vless:// ссылку",
        "vpn.sub.default_name":   "Montana (по умолчанию)",
        "vpn.sub.custom":         "Пользовательская",
        "vpn.section.auto":       "Автоматическое",
        "vpn.section.manual":     "Ручные серверы",

        // Profile / Settings
        "profile.title":          "Профиль",
        "profile.settings":       "Настройки",
        "settings.title":         "Настройки приложения",
        "settings.language":      "Язык интерфейса",
        "settings.autostart_app": "Запускать Montana при входе в систему",
        "settings.autostart_node": "Запускать узел автоматически",
        "settings.autostart_vpn": "Подключать ВПН автоматически",
        "settings.refresh_min":   "Период авто-обновления подписок (мин)",
        "settings.killswitch":    "Kill switch (без ВПН — нет интернета)",
        "settings.killswitch_hint": "Если ВПН-соединение оборвётся — системный прокси останется включён, и трафик не пойдёт в обход.",
        "settings.about":         "О приложении",
        "settings.about.body":    "Montana Ядро 0.1 — полный узел постквантового мейннета Монтаны. Открытый исходный код: github.com/efir369999/Montana",
        "settings.done":          "Готово",

    ]

    // MARK: - EN
    static let en: [String: String] = [
        "tab.wallet": "Wallet", "tab.vpn": "VPN", "tab.network": "Network",
        "side.status": "status",
        "side.node.stopped": "node: stopped",
        "side.node.running": "node: %@",
        "side.net.window": "net: window %@",
        "side.vpn.on": "VPN: %@",
        "side.version": "Montana Core 0.1",

        "onboard.title": "Welcome to Montana",
        "onboard.subtitle": "This is a full Montana node. Installing this app = your laptop becomes part of mainnet.",
        "onboard.step1.t": "Create the key",
        "onboard.step1.b": "24 words — the only way to recover the wallet. Write on paper, show no one.",
        "onboard.step2.t": "The node starts",
        "onboard.step2.b": "Local montana-node process starts in background. Connects to network peers and syncs current state.",
        "onboard.step3.t": "The node computes VDF",
        "onboard.step3.b": "A post-quantum \"timer\" proves you spent τ₂ = 20,160 windows in the network. You can close the laptop — it resumes from the same window next launch.",
        "onboard.step4.t": "You register as a validator",
        "onboard.step4.b": "After τ₂ = 20,160 windows the node submits its claim. At the next selection window (every 336 windows) you are accepted.",
        "onboard.step5.t": "You earn emission",
        "onboard.step5.b": "An Active validator earns 13 Ɉ per window while the node runs.",
        "onboard.lifespan.t": "The node lives while Montana is running",
        "onboard.lifespan.b": "Minimize → keeps running. Close → stops. Next open resumes from same VDF window.",
        "onboard.btn.create": "Create wallet and start node",
        "onboard.recover": "Already have 24 words — recover",
        "onboard.words": "%d of 24",
        "onboard.btn.restore": "Restore",

        "mnemonic.warn": "Write down 24 words. Without them you lose wallet access. After \"Saved\" they cannot be shown again.",
        "mnemonic.copy": "Copy",
        "mnemonic.saved": "Saved — start node",
        "mnemonic.copied": "copied",

        "wallet.title": "Wallet",
        "wallet.subtitle": "Montana node (local)",
        "wallet.balance": "balance",
        "wallet.crypto.label": "crypto",
        "wallet.crypto.val": "ML-DSA-65",
        "wallet.account_id": "your account_id",
        "wallet.send": "Send",
        "wallet.receive": "Receive",
        "wallet.wipe": "Delete wallet and node",
        "wallet.node_stopped": "Node stopped",
        "wallet.node_off_note": "To receive balance updates and emission — the node must run. First launch: wait until τ₂ = 20,160 windows.",
        "wallet.start_node": "Start node",
        "wallet.candidate": "Candidate progress",
        "wallet.candidate_note": "Upon reaching τ₂ = 20,160 windows the node submits its registration claim.",
        "wallet.candidate_of": "%@ of %@ windows",

        "wipe.title": "Delete wallet?",
        "wipe.info": "This deletes the local node identity. Cannot recover without the 24 words.",
        "wipe.ok": "Delete",
        "wipe.cancel": "Cancel",

        "net.title": "Montana Network",
        "net.subtitle": "Mainnet",
        "net.synced": "synced with network",
        "net.summary": "Active nodes %d of %d; external operators: %d.",
        "net.no_link": "no network link",
        "net.my_node": "Your node in the network",
        "net.synced_to": "synced: %@ of %@",
        "net.not_started": "Node not running. Go to Wallet tab to create or start it.",
        "net.connecting": "connecting…",
        "net.supply": "supply",
        "net.window": "window",
        "net.nodes": "nodes",
        "net.active_total": "active/total",
        "net.explorer": "Explorer",
        "net.last_update": "updated %@",
        "net.install_node": "Anyone can deploy a Montana node by running on a clean Linux VPS:",

        "phase.unknown":      "not running",
        "phase.bootstrap":    "connecting to network",
        "phase.candidateVdf": "candidate · VDF",
        "phase.registered":   "awaiting admission",
        "phase.active":       "active validator",

        "vpn.title": "Montana VPN",
        "vpn.state.off": "off",
        "vpn.state.conn": "connecting…",
        "vpn.state.on": "connected",
        "vpn.state.err": "error",
        "vpn.servers": "Servers",
        "vpn.connect": "Connect",
        "vpn.disconnect": "Disconnect",
        "vpn.loading": "loading servers…",
        "vpn.connecting_to": "starting xray → %@…",
        "vpn.connected_to": "connected to %@ %@",
        "vpn.disconnecting": "disconnecting…",
        "vpn.error": "error",
        "vpn.error_label": "error: %@",

        "menu.site": "Montana site",
        "menu.explorer": "Explorer",        "vpn.port_busy": "Port 10808 is used by another VPN client (e.g. Happ). Close it so Montana can manage the connection.",
        "vpn.first_setup.title": "Montana — one-time setup",
        "vpn.first_setup.body": "To route traffic through Montana nodes, you must allow system proxy management once. macOS will now ask for your admin password. After that, VPN turns on and off instantly, with no prompts.",
        "vpn.first_setup.ok": "Allow",
        "vpn.first_setup.cancel": "Cancel",

        // VPN extended
        "vpn.subs":               "Subscriptions",
        "vpn.sub.add":            "Add subscription",
        "vpn.sub.url_ph":         "Subscription URL or vless://",
        "vpn.sub.refresh":        "Refresh",
        "vpn.sub.refresh_all":    "Refresh all",
        "vpn.sub.delete":         "Delete",
        "vpn.sub.last_update":    "updated: %@",
        "vpn.sub.never":          "never",
        "vpn.sub.servers_count":  "%d servers",
        "vpn.sub.auto":           "Auto-refresh every %d min",
        "vpn.ping_all":           "Ping all",
        "vpn.add_server":         "Add server",
        "vpn.server.name_ph":     "Name (optional)",
        "vpn.server.url_ph":      "vless://… (paste URL)",
        "vpn.server.add":         "Add",
        "vpn.server.cancel":      "Cancel",
        "vpn.empty":              "No servers — add a subscription or paste a vless:// URL",
        "vpn.sub.default_name":   "Montana (default)",
        "vpn.sub.custom":         "Custom",
        "vpn.section.auto":       "Automatic",
        "vpn.section.manual":     "Manual servers",

        // Profile / Settings
        "profile.title":          "Profile",
        "profile.settings":       "Settings",
        "settings.title":         "App settings",
        "settings.language":      "Interface language",
        "settings.autostart_app": "Launch Montana at login",
        "settings.autostart_node": "Auto-start node",
        "settings.autostart_vpn": "Auto-connect VPN",
        "settings.refresh_min":   "Subscription auto-refresh interval (min)",
        "settings.killswitch":    "Kill switch (no VPN — no internet)",
        "settings.killswitch_hint": "If VPN connection drops — system proxy stays enabled, traffic won't leak around it.",
        "settings.about":         "About",
        "settings.about.body":    "Montana Core 0.1 — full node of the post-quantum Montana mainnet. Open source: github.com/efir369999/Montana",
        "settings.done":          "Done",

    ]

    // MARK: - ZH
    static let zh: [String: String] = [
        "tab.wallet": "钱包", "tab.vpn": "VPN", "tab.network": "网络",
        "side.status": "状态",
        "side.node.stopped": "节点：已停止",
        "side.node.running": "节点：%@",
        "side.net.window": "网络：窗口 %@",
        "side.vpn.on": "VPN：%@",
        "side.version": "Montana 内核 0.1",

        "onboard.title": "欢迎来到 Montana",
        "onboard.subtitle": "这是一个完整的 Montana 节点。安装此应用 = 您的笔记本成为主网的一部分。",
        "onboard.step1.t": "创建密钥",
        "onboard.step1.b": "24 个单词 — 恢复钱包的唯一方法。请记在纸上，不要给任何人看。",
        "onboard.step2.t": "节点启动",
        "onboard.step2.b": "本地 montana-node 进程在后台启动。连接到网络节点并同步当前状态。",
        "onboard.step3.t": "节点计算 VDF",
        "onboard.step3.b": "后量子\"计时器\"证明您在网络中度过了 τ₂ = 20,160 个窗口。可以关闭笔记本 — 下次启动时从同一个窗口继续。",
        "onboard.step4.t": "注册为验证者",
        "onboard.step4.b": "在 τ₂ = 20,160 个窗口后，节点提交其声明。在下一个选择窗口（每 336 个窗口）您被接受。",
        "onboard.step5.t": "获得发行",
        "onboard.step5.b": "活跃验证者在节点运行时每个窗口获得 13 Ɉ。",
        "onboard.lifespan.t": "节点在 Montana 应用运行时存活",
        "onboard.lifespan.b": "最小化 → 继续运行。关闭 → 停止。下次打开从相同 VDF 窗口继续。",
        "onboard.btn.create": "创建钱包并启动节点",
        "onboard.recover": "已有 24 个单词 — 恢复",
        "onboard.words": "%d / 24",
        "onboard.btn.restore": "恢复",

        "mnemonic.warn": "写下 24 个单词。没有它们您将失去钱包访问权限。点击「已保存」后无法再次显示。",
        "mnemonic.copy": "复制",
        "mnemonic.saved": "已保存 — 启动节点",
        "mnemonic.copied": "已复制",

        "wallet.title": "钱包",
        "wallet.subtitle": "Montana 节点（本地）",
        "wallet.balance": "余额",
        "wallet.crypto.label": "密码体系",
        "wallet.crypto.val": "ML-DSA-65",
        "wallet.account_id": "您的 account_id",
        "wallet.send": "发送",
        "wallet.receive": "接收",
        "wallet.wipe": "删除钱包和节点",
        "wallet.node_stopped": "节点已停止",
        "wallet.node_off_note": "要接收余额更新和发行 — 节点必须运行。首次启动：等待 τ₂ = 20,160 个窗口。",
        "wallet.start_node": "启动节点",
        "wallet.candidate": "候选进度",
        "wallet.candidate_note": "达到 τ₂ = 20,160 个窗口时节点提交其注册声明。",
        "wallet.candidate_of": "%@ / %@ 个窗口",

        "wipe.title": "删除钱包？",
        "wipe.info": "这将删除本地节点身份。没有 24 个单词无法恢复。",
        "wipe.ok": "删除",
        "wipe.cancel": "取消",

        "net.title": "Montana 网络",
        "net.subtitle": "主网",
        "net.synced": "已与网络同步",
        "net.summary": "活跃节点 %d / %d；外部操作员：%d。",
        "net.no_link": "无网络链接",
        "net.my_node": "您在网络中的节点",
        "net.synced_to": "已同步：%@ / %@",
        "net.not_started": "节点未运行。前往钱包标签创建或启动。",
        "net.connecting": "连接中…",
        "net.supply": "总量",
        "net.window": "窗口",
        "net.nodes": "节点",
        "net.active_total": "活跃/总数",
        "net.explorer": "浏览器",
        "net.last_update": "更新于 %@",
        "net.install_node": "任何人都可以通过在干净的 Linux VPS 上运行以下命令来部署 Montana 节点：",

        "phase.unknown":      "未运行",
        "phase.bootstrap":    "正在连接网络",
        "phase.candidateVdf": "候选 · VDF",
        "phase.registered":   "等待录取",
        "phase.active":       "活跃验证者",

        "vpn.title": "Montana VPN",
        "vpn.state.off": "已关闭",
        "vpn.state.conn": "连接中…",
        "vpn.state.on": "已连接",
        "vpn.state.err": "错误",
        "vpn.servers": "服务器",
        "vpn.connect": "连接",
        "vpn.disconnect": "断开",
        "vpn.loading": "正在加载服务器…",
        "vpn.connecting_to": "启动 xray → %@…",
        "vpn.connected_to": "已连接到 %@ %@",
        "vpn.disconnecting": "断开中…",
        "vpn.error": "错误",
        "vpn.error_label": "错误：%@",

        "menu.site": "Montana 站点",
        "menu.explorer": "浏览器",        "vpn.port_busy": "端口 10808 被另一个 VPN 客户端（如 Happ）占用。请关闭它，以便 Montana 管理连接。",
        "vpn.first_setup.title": "Montana — 一次性设置",
        "vpn.first_setup.body": "要通过 Montana 节点路由流量，需要授权一次系统代理管理。macOS 现在会要求输入管理员密码。之后，VPN 将立即开启和关闭，无需提示。",
        "vpn.first_setup.ok": "允许",
        "vpn.first_setup.cancel": "取消",

        // VPN extended
        "vpn.subs":               "订阅",
        "vpn.sub.add":            "添加订阅",
        "vpn.sub.url_ph":         "订阅 URL 或 vless://",
        "vpn.sub.refresh":        "刷新",
        "vpn.sub.refresh_all":    "全部刷新",
        "vpn.sub.delete":         "删除",
        "vpn.sub.last_update":    "更新于：%@",
        "vpn.sub.never":          "从未",
        "vpn.sub.servers_count":  "%d 个服务器",
        "vpn.sub.auto":           "每 %d 分钟自动刷新",
        "vpn.ping_all":           "测试全部 ping",
        "vpn.add_server":         "添加服务器",
        "vpn.server.name_ph":     "名称（可选）",
        "vpn.server.url_ph":      "vless://…（粘贴 URL）",
        "vpn.server.add":         "添加",
        "vpn.server.cancel":      "取消",
        "vpn.empty":              "没有服务器 — 添加订阅或粘贴 vless:// 链接",
        "vpn.sub.default_name":   "Montana（默认）",
        "vpn.sub.custom":         "自定义",
        "vpn.section.auto":       "自动",
        "vpn.section.manual":     "手动服务器",

        // Profile / Settings
        "profile.title":          "个人资料",
        "profile.settings":       "设置",
        "settings.title":         "应用设置",
        "settings.language":      "界面语言",
        "settings.autostart_app": "登录时启动 Montana",
        "settings.autostart_node": "自动启动节点",
        "settings.autostart_vpn": "自动连接 VPN",
        "settings.refresh_min":   "订阅自动刷新间隔（分钟）",
        "settings.killswitch":    "Kill switch（无 VPN — 无网络）",
        "settings.killswitch_hint": "如果 VPN 连接断开 — 系统代理保持启用，流量不会绕过它。",
        "settings.about":         "关于",
        "settings.about.body":    "Montana 内核 0.1 — 后量子 Montana 主网的完整节点。开源：github.com/efir369999/Montana",
        "settings.done":          "完成",

    ]
}
