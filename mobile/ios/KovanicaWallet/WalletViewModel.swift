import Foundation
import SwiftUI
import Combine

@MainActor
class WalletViewModel: ObservableObject {
    @Published var isWalletReady = false
    @Published var isLoading = false
    @Published var errorMessage: String?
    @Published var walletData: WalletData?
    @Published var transactions: [Transaction] = []
    @Published var headData: HeadData?
    @Published var authorities: [Authority] = []
    @Published var sendResult: String?

    private let apiBase = "https://explorer.kovanica.online"
    private var cancellables = Set<AnyCancellable>()

    func initializeWallet(phrase: String, passphrase: String) {
        isLoading = true
        errorMessage = nil

        // Derive address from phrase (simplified - would use BIP39 in production)
        let address = deriveAddress(from: phrase)

        walletData = WalletData(
            address: address,
            balance: "0",
            chainHeight: 0,
            blockCount: 0,
            selectedTip: "",
            isSyncing: true
        )

        isWalletReady = true
        isLoading = false

        Task {
            await refresh()
        }
    }

    func refresh() async {
        guard let wallet = walletData else { return }

        isLoading = true
        defer { isLoading = false }

        do {
            // Fetch head
            if let head = try? await fetchHead() {
                headData = head
                walletData = WalletData(
                    address: wallet.address,
                    balance: wallet.balance,
                    chainHeight: head.chainHeight,
                    blockCount: head.blockCount,
                    selectedTip: head.selectedTip,
                    isSyncing: false
                )
            }

            // Fetch balance
            if let balance = try? await fetchBalance(address: wallet.address) {
                walletData = WalletData(
                    address: wallet.address,
                    balance: balance,
                    chainHeight: walletData?.chainHeight ?? 0,
                    blockCount: walletData?.blockCount ?? 0,
                    selectedTip: walletData?.selectedTip ?? "",
                    isSyncing: false
                )
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    func send(to address: String, amount: String) async {
        isLoading = true
        defer { isLoading = false }

        // In production: prepare → sign → submit
        sendResult = "Transaction prepared (signing required)"
    }

    func loadHistory() async {
        guard let wallet = walletData else { return }
        // Fetch history from API
    }

    // MARK: - Private

    private func deriveAddress(from phrase: String) -> String {
        // Simplified - would use BIP39 + Ed25519 in production
        let hash = phrase.data(using: .utf8)?.base64EncodedString() ?? ""
        return "kvnc1" + String(hash.prefix(38))
    }

    private func fetchHead() async throws -> HeadData {
        guard let url = URL(string: "\(apiBase)/api/head") else {
            throw URLError(.badURL)
        }
        let (data, _) = try await URLSession.shared.data(from: url)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? []

        return HeadData(
            genesis: json["genesis"] as? String ?? "",
            selectedTip: json["selected_tip"] as? String ?? "",
            blockCount: json["block_count"] as? Int ?? 0,
            chainHeight: json["chain_height"] as? Int ?? 0,
            maxSupply: json["max_supply"] as? String ?? "0",
            minted: json["minted"] as? String ?? "0",
            minFee: json["min_fee"] as? Int ?? 0,
            finalityDepth: json["finality_depth"] as? Int ?? 0,
            era: json["era"] as? Int ?? 0,
            eraLen: json["era_len"] as? Int ?? 0,
            subsidy: json["subsidy"] as? String ?? "0"
        )
    }

    private func fetchBalance(address: String) async throws -> String {
        guard let url = URL(string: "\(apiBase)/api/balance/\(address)") else {
            throw URLError(.badURL)
        }
        let (data, _) = try await URLSession.shared.data(from: url)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? []
        return json["balance"] as? String ?? "0"
    }
}
