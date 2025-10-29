#!/usr/bin/env node
import { run, hadError } from './index.ts'
import process from 'node:process'
import fs from 'node:fs'
import readline from 'node:readline'

if (process.argv.length > 3) {
	console.error('Usage: loxjs [file.lox]')
	process.exitCode = 64
} else if (process.argv.length === 3) {
	run(fs.readFileSync(process.argv[2], 'utf-8'))
	if (hadError) process.exitCode = 65
} else {
	const rl = readline.createInterface(process.stdin, process.stdout)
	rl.setPrompt('> ')
	rl.prompt()
	rl.on('line', line => {
		run(line)
		rl.prompt()
	})
}
