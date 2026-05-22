// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "MontanaMonitor",
    platforms: [.macOS(.v14)],
    targets: [
        .executableTarget(
            name: "MontanaMonitor",
            path: ".",
            exclude: ["build.sh", "quest.montana.monitor.plist", ".build", "MontanaMonitor.app"],
            sources: ["MontanaMonitor.swift"]
        )
    ]
)
