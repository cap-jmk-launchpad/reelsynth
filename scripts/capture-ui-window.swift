import Cocoa
import CoreGraphics

let opts = CGWindowListOption(arrayLiteral: .optionOnScreenOnly, .excludeDesktopElements)
guard let list = CGWindowListCopyWindowInfo(opts, kCGNullWindowID) as? [[String: Any]] else {
    fputs("no windows\n", stderr)
    exit(1)
}

let needle = CommandLine.arguments.count > 1 ? CommandLine.arguments[1].lowercased() : "reelsynth"
var found: Int?

for w in list {
    let owner = (w[kCGWindowOwnerName as String] as? String ?? "").lowercased()
    let name = (w[kCGWindowName as String] as? String ?? "").lowercased()
    let layer = w[kCGWindowLayer as String] as? Int ?? 0
    guard layer == 0 else { continue }
    if owner.contains(needle) || name.contains(needle) {
        if let id = w[kCGWindowNumber as String] as? Int {
            found = id
            if let bounds = w[kCGWindowBounds as String] {
                print("bounds=\(bounds)")
            }
            print("id=\(id)")
            break
        }
    }
}

if found == nil {
    fputs("window not found for \(needle)\n", stderr)
    exit(2)
}
