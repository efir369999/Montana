import SwiftUI

@main
struct MontanaApp: App {
    var body: some Scene {
        WindowGroup("Montana") {
            ContentView()
        }
        .defaultSize(width: 760, height: 660)
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(replacing: .newItem) { }
        }
    }
}
