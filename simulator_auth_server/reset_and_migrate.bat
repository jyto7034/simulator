@echo off
chcp 65001 >nul
echo ========================================
echo PostgreSQL Database Reset and Migration
echo ========================================

set PGPASSWORD=root
set DB_USER=postgres
set DB_HOST=localhost
set DB_PORT=5432
set DB_NAME=auth
set DEFAULT_DB=postgres
set PGCLIENTENCODING=UTF8

echo Dropping database '%DB_NAME%'...
psql -h %DB_HOST% -p %DB_PORT% -U %DB_USER% -d %DEFAULT_DB% -c "DROP DATABASE IF EXISTS %DB_NAME%;"

if %errorlevel% neq 0 (
    echo Error: Failed to drop database
    pause
    exit /b 1
)

echo Creating database '%DB_NAME%'...
psql -h %DB_HOST% -p %DB_PORT% -U %DB_USER% -d %DEFAULT_DB% -c "CREATE DATABASE %DB_NAME%;"

if %errorlevel% neq 0 (
    echo Error: Failed to create database
    pause
    exit /b 1
)

echo Running migrations...
for %%f in (migrations\0*.sql) do (
    echo Applying migration: %%f
    psql -h %DB_HOST% -p %DB_PORT% -U %DB_USER% -d %DB_NAME% -f "%%f"
    if %errorlevel% neq 0 (
        echo Error: Failed to apply migration %%f
        pause
        exit /b 1
    )
)

echo ========================================
echo Database reset and migration completed successfully!
echo ========================================
pause