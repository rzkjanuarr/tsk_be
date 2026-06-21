#!/bin/bash

echo "🔍 Checking DATABASE_URL..."
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL not set!"
    echo ""
    echo "Set it with:"
    echo "export DATABASE_URL='postgresql://postgres:CHFihdhoRPHvOqJoxEajpvcbctaRdrWo@postgres-production-198c.up.railway.app:5432/railway'"
    exit 1
else
    echo "✅ DATABASE_URL is set"
fi

echo ""
echo "🚀 Running backend with debug info..."
export RUST_BACKTRACE=1
cargo run --release
