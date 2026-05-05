// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "Montana",
    platforms: [.macOS(.v14)],
    targets: [
        .executableTarget(
            name: "Montana",
            path: "Sources/Montana",
            resources: [
                .process("Resources")
            ],
            swiftSettings: [
                .unsafeFlags(["-parse-as-library"])
            ]
        )
    ]
)
