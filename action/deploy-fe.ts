const cmd = "km run -y action deploy-komodo-fe-change";
new Deno.Command("bash", {
  args: ["-c", cmd],
}).spawn();