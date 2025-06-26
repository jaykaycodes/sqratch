import { debug, error, info, trace, warn } from '@tauri-apps/plugin-log'

type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'trace'
type LogFn = (message: string) => Promise<void>
type LogData = Record<string, any>

class Logger {
	private formatMessage(message: string, data?: LogData): string {
		if (!data || Object.keys(data).length === 0) {
			return message
		}

		const formattedData = Object.entries(data)
			.map(([key, value]) => `${key}=${typeof value === 'object' ? JSON.stringify(value) : value}`)
			.join(' ')

		return `${message} ${formattedData}`
	}

	private createLogMethod(level: LogLevel, logFn: LogFn) {
		const original = console[level]

		return (message: string, ...args: any[]) => {
			// Extract object properties from the rest parameters
			const data =
				args.length === 1 && typeof args[0] === 'object'
					? args[0]
					: args.length > 0
						? Object.fromEntries(
								args
									.filter((_, i) => i % 2 === 0)
									.map((key, i) => [key, args[i * 2 + 1]])
									.filter(([_, v]) => v !== undefined),
							)
						: undefined

			const formattedMessage = this.formatMessage(message, data)
			original(formattedMessage)
			logFn(formattedMessage).catch(console.error)
		}
	}

	debug = this.createLogMethod('debug', debug)
	info = this.createLogMethod('info', info)
	warn = this.createLogMethod('warn', warn)
	error = this.createLogMethod('error', error)
	trace = this.createLogMethod('trace', trace)
}

const logger = new Logger()

export default logger
