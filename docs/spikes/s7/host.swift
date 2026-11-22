// Spike S7 host: a WKWebView with a custom scheme (like Tauri's tauri://localhost) serving
// ./www with a CSP header, printing what the page reports.
import Cocoa
import WebKit

let root = URL(fileURLWithPath: CommandLine.arguments[1])
let csp = CommandLine.arguments[2]
let page = CommandLine.arguments.count > 3 ? CommandLine.arguments[3] : "index.html"

class Scheme: NSObject, WKURLSchemeHandler {
    let policy: String
    init(_ p: String) { policy = p }
    func webView(_ w: WKWebView, start task: WKURLSchemeTask) {
        let path = task.request.url!.path
        let file = root.appendingPathComponent(path == "/" ? "index.html" : String(path.dropFirst()))
        guard let data = try? Data(contentsOf: file) else {
            task.didFailWithError(NSError(domain: "s7", code: 404)); return
        }
        let mime = file.pathExtension == "html" ? "text/html" : file.pathExtension == "js" ? "text/javascript" : "application/octet-stream"
        var headers = ["Content-Type": mime]
        if file.pathExtension == "html" { headers["Content-Security-Policy"] = policy }
        let r = HTTPURLResponse(url: task.request.url!, statusCode: 200, httpVersion: "HTTP/1.1", headerFields: headers)!
        task.didReceive(r); task.didReceive(data); task.didFinish()
    }
    func webView(_ w: WKWebView, stop task: WKURLSchemeTask) {}
}

class Done: NSObject, WKScriptMessageHandler {
    func userContentController(_ c: WKUserContentController, didReceive m: WKScriptMessage) {
        if let s = m.body as? String {
            print(s)
            if s.hasPrefix("DONE") { exit(0) }
        }
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.prohibited)
let cfg = WKWebViewConfiguration()
cfg.setURLSchemeHandler(Scheme(csp), forURLScheme: "cruncha")
cfg.setURLSchemeHandler(Scheme(CommandLine.arguments.count > 4 ? CommandLine.arguments[4] : csp), forURLScheme: "crunchafig")
cfg.userContentController.add(Done(), name: "log")
let win = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1000, height: 800), styleMask: [.borderless], backing: .buffered, defer: false)
let web = WKWebView(frame: win.contentView!.bounds, configuration: cfg)
win.contentView!.addSubview(web)
win.orderFrontRegardless()
web.load(URLRequest(url: URL(string: "cruncha://localhost/\(page)")!))
DispatchQueue.main.asyncAfter(deadline: .now() + 150) { print("DONE timeout"); exit(1) }
app.run()
