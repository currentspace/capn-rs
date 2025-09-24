#!/usr/bin/env node

// src/test-newline-format.ts
async function testNewlineFormat() {
  console.log("\u{1F50D} Testing newline-delimited format (official Cap'n Web)\n");
  const body = '["push",["import",0,["add"],[5,3]]]';
  console.log("\u{1F4E4} Sending (newline-delimited, no Content-Type):");
  console.log(body);
  try {
    const response = await fetch("http://localhost:8080/rpc/batch", {
      method: "POST",
      // NO Content-Type header (like official client)
      body
    });
    console.log("\n\u{1F4E5} Response:");
    console.log(`Status: ${response.status} ${response.statusText}`);
    console.log("Headers:", Object.fromEntries(response.headers.entries()));
    const responseBody = await response.text();
    console.log("Body:", responseBody);
    console.log("Body length:", responseBody.length);
    if (responseBody === "") {
      console.log("\n\u2705 Got empty response (expected for Push without Pull)");
    } else {
      console.log("\n\u{1F4DD} Parsing newline-delimited response:");
      const lines = responseBody.split("\n").filter((line) => line.trim());
      for (const line of lines) {
        console.log("  Line:", line);
        try {
          const parsed = JSON.parse(line);
          console.log("  Parsed:", JSON.stringify(parsed));
        } catch (e) {
          console.log("  Parse error:", e);
        }
      }
    }
  } catch (error) {
    console.error("Request failed:", error);
  }
}
testNewlineFormat();
//# sourceMappingURL=test-newline-format.js.map