export const run = async (action: string) => {
  const branch = await new Deno.Command("bash", {
    args: ["-c", "git rev-parse --abbrev-ref HEAD"],
  })
    .output()
    .then((r) => new TextDecoder("utf-8").decode(r.stdout).trim());

  new Deno.Command("bash", {
    args: ["-c", `km run -y action ${action} "BRANCH=${branch}"`],
  }).spawn();
};
