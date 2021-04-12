import execa from "execa";

export const shell = async (
  command: string,
  args?: string[],
  options?: execa.Options,
  log: boolean = false
) => {
  try {
    if (options && options.shell === true) {
      const stringCommand = [command, ...(!args ? [] : args)].join(" ");
      if (log) console.log(`[running]: ${stringCommand}`);
      return await execa(stringCommand, {
        stdio: "inherit",
        cwd: process.cwd(),
        env: process.env,
        ...options,
      });
    } else {
      if (log) console.log(`[running]: ${command}`);
      return await execa(command, args, {
        stdio: "inherit",
        cwd: process.cwd(),
        env: process.env,
        ...options,
      });
    }
  } catch (error) {
    console.error("Error with command: %s", command);
    throw new Error(error);
  }
};
