// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "MontanaMonitor",
    platforms: [.macOS(.v14)],
    targets: [
        .systemLibrary(
            name: "MontanaBindings",
            path: "Sources/MontanaBindings"
        ),
        .executableTarget(
            name: "MontanaMonitor",
            dependencies: ["MontanaBindings"],
            path: "Sources/MontanaMonitor",
            linkerSettings: [
                .unsafeFlags(["-L", "Resources/mt-bindings", "-lmt_bindings"])
            ]
        )
    ]
)
