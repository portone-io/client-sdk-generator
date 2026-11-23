// Generate this fixture's models with the Swift backend, then compile them
// with this file and run the resulting executable.
import Foundation

@main
struct NativeObjectUnionSmoke {
    static func main() throws {
        let decoder = JSONDecoder()
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]

        func decode(_ json: String) throws -> SyntheticRequest {
            try decoder.decode(SyntheticRequest.self, from: Data(json.utf8))
        }

        func encode(_ request: SyntheticRequest) throws -> String {
            String(decoding: try encoder.encode(request), as: UTF8.self)
        }

        let absent = try decode("{}")
        precondition(absent.bypass == nil)
        let absentJSON = try encode(absent)
        precondition(absentJSON == "{}")

        let empty = try decode(#"{"bypass":{}}"#)
        guard case .choiceA? = empty.bypass else {
            fatalError("Identical branches should choose the first declared case")
        }
        let emptyJSON = try encode(empty)
        precondition(emptyJSON == #"{"bypass":{}}"#)

        let first = try decode(#"{"bypass":{"shared_pg":{"logo_url":""}}}"#)
        guard case .choiceA(let value)? = first.bypass else {
            fatalError("Expected first branch")
        }
        precondition(value.sharedPg?.logoUrl == "")
        let firstJSON = try encode(first)
        precondition(firstJSON == #"{"bypass":{"shared_pg":{"logo_url":""}}}"#)

        let second = try decode(#"{"bypass":{"shared_pg":{"count":0,"modes":[],"opt_in":false}}}"#)
        guard case .choiceB(let value)? = second.bypass else {
            fatalError("Nested keys should match the second branch even when the first also decodes")
        }
        precondition(value.sharedPg?.count == 0)
        precondition(value.sharedPg?.modes == [])
        precondition(value.sharedPg?.optIn == false)
        let secondJSON = try encode(second)
        precondition(secondJSON == #"{"bypass":{"shared_pg":{"count":0,"modes":[],"opt_in":false}}}"#)

        let shared = try decode(#"{"bypass":{"both":{"item_code":""}}}"#)
        guard case .choiceA? = shared.bypass else {
            fatalError("Equivalent payloads should choose the first declared case")
        }
        let sharedJSON = try encode(shared)
        precondition(sharedJSON == #"{"bypass":{"both":{"item_code":""}}}"#)

        for invalid in [
            #"{"bypass":{"shared_pg":{"opt_in":"false"}}}"#,
            #"{"bypass":{"shared_pg":{"logo_url":"one","count":1}}}"#,
            #"{"bypass":{"unrecognized":{}}}"#,
            #"{"bypass":{"shared_pg":{"logo_url":null}}}"#,
            #"{"bypass":[]}"#,
        ] {
            do {
                _ = try decode(invalid)
                fatalError("Invalid union unexpectedly decoded: \(invalid)")
            } catch is DecodingError {
                // A partial decode would otherwise silently throw away unknown fields.
            }
        }
    }
}
