import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

export let useSshTesting = () => {
  let [testStatus, setTestStatus] = useState<"idle" | "testing" | "success" | "error">("idle")
  let [testMessage, setTestMessage] = useState("")

  let testSshConnection = async (sshHost: string) => {
    if (!sshHost) {
      setTestStatus("error")
      setTestMessage("Please enter SSH host")
      return
    }

    setTestStatus("testing")
    setTestMessage("Testing SSH connection...")

    try {
      let result = await invoke<string>("test_ssh_connection", { sshHost })
      setTestStatus("success")
      setTestMessage(result)
    } catch (error) {
      setTestStatus("error")
      setTestMessage(error as string)
    }
  }

  let resetTest = () => {
    setTestStatus("idle")
    setTestMessage("")
  }

  return {
    testStatus,
    testMessage,
    testSshConnection,
    resetTest,
  }
}