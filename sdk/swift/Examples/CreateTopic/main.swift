import Hedera
import SwiftDotenv

@main
public enum Program {
    public static func main() async throws {
        let env = try Dotenv.load()
        let client = try Client.forName(env.networkName)

        // Defaults the operator account ID and key such that all generated transactions will be paid for
        // by this account and be signed by this key
        client.setOperator(env.operatorAccountId, env.operatorKey)

        let createResponse = try await TopicCreateTransaction().execute(client)
        let createReceipt = try await createResponse.getReceipt(client)

        print("topic id = \(createReceipt.topicId!)")

        let sendResponse = try await TopicMessageSubmitTransaction()
            .topicId(createReceipt.topicId!)
            .message("hello world".data(using: .utf8)!)
            .execute(client)

        let sendReceipt = try await sendResponse.getReceipt(client)

        print("sequence number = \(sendReceipt.topicSequenceNumber)")
    }
}
extension Environment {
    public var operatorAccountId: AccountId {
        AccountId(self["OPERATOR_ACCOUNT_ID"]!.stringValue)!
    }

    public var operatorKey: PrivateKey {
        PrivateKey(self["OPERATOR_KEY"]!.stringValue)!
    }

    public var networkName: String {
        self["HEDERA_NETWORK"]?.stringValue ?? "testnet"
    }
}
