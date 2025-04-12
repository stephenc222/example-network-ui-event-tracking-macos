//
//  ContentView.swift
//  example-mac-app
//
//  Created by Stephen Collins on 4/8/25.
//

import SwiftUI

struct ContentView: View {
    @State private var responseText: String = "Click a button to fetch todo info"

    var body: some View {
        VStack(spacing: 20) {
            Text("Example Mac App")
                .font(.title)
                .fontWeight(.bold)
                .accessibilityLabel("Example Mac App") // 👈 This makes the title discoverable
                .padding(.top, 10)

            Button("Button A") {
                fetchProduct(id: 1)
            }
            .accessibilityIdentifier("ButtonA")

            Button("Button B") {
                fetchProduct(id: 2)
            }
            .accessibilityIdentifier("ButtonB")

            Button("Button C") {
                fetchProduct(id: 3)
            }
            .accessibilityIdentifier("ButtonC")

            Divider()

            ScrollView {
                Text(responseText)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()
                    .font(.system(.body, design: .monospaced))
            }
            .frame(maxHeight: 200)
            .border(Color.gray)
        }
        .padding()
    }

    func fetchProduct(id: Int) {
        responseText = "Loading product \(id)..."

        guard let url = URL(string: "https://jsonplaceholder.typicode.com/todos/\(id)") else {
            responseText = "Invalid URL"
            return
        }

        let task = URLSession.shared.dataTask(with: url) { data, response, error in
            DispatchQueue.main.async {
                if let error = error {
                    responseText = "Error fetching product \(id): \(error)"
                    return
                }

                if let data = data, let responseStr = String(data: data, encoding: .utf8) {
                    responseText = responseStr
                } else {
                    responseText = "No data received for product \(id)"
                }
            }
        }

        task.resume()
    }
}
