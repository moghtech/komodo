const cmd = "km run -y action deploy-komodo-ui-change";
new Deno.Command("bash", {
  args: ["-c", cmd],
}).spawn();