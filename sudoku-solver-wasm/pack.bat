@echo off
wasm-pack build --target no-modules
if %errorlevel% neq 0 exit /b %errorlevel%
copy ..\user-scripts\fpuzzles-sudokusolver-wasm-worker.js .\node\public
copy .\pkg\sudoku_solver_wasm.d.ts .\node\public
copy .\pkg\sudoku_solver_wasm.js .\node\public
copy .\pkg\sudoku_solver_wasm_bg.wasm .\node\public
copy .\pkg\sudoku_solver_wasm_bg.wasm.d.ts .\node\public