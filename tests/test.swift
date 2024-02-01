import Foundation

func test() throws {
    print("HELLO WORLD from swift")

    let session = URLSession.shared
    
    struct HTTPClient: NetworkRequestMaker {
        func makeRequest(request: NetworkRequest)
    }
}

try! test()