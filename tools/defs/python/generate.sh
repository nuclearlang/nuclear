#!/bin/sh
# Regenerates defs/python/: the standard library here and the packages of the fastapi dialect in
# dragonfly-backend/defs/python/ (plan 006), with the
# versions installed in tools/python/.venv (tools/python/requirements.txt). Then update the `defs` lists of
# dialects/python and dialects/backend/fastapi if a version changed, and run `cargo test`.
set -e
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
py="$root/tools/python/.venv/bin/python"
out="$root/defs"
# the packages of the fastapi dialect live in the backend platform's repository (plan 006); the standard library here
platform="${DRAGONFLY_BACKEND:-$root/dragonfly-backend}/defs"
v() { "$py" -c "import importlib.metadata as m; print(m.version('$1'))"; }
gen() { "$py" "$here/gen.py" "$@"; }

pyv=$("$py" -c 'import sys; print(f"{sys.version_info[0]}.{sys.version_info[1]}")')
gen python "$pyv" "$out" builtins typing dataclasses enum abc collections collections.abc functools itertools \
  datetime time json re math random decimal uuid pathlib os logging asyncio contextlib \
  --implicit builtins --description "Python $pyv standard library (builtins implicit)"
P="python@$pyv"
gen pydantic "$(v pydantic)" "$platform" pydantic --depends "$P" --description "pydantic: data validation with type hints"
gen pydantic_settings "$(v pydantic-settings)" "$platform" pydantic_settings --depends "$P,pydantic@$(v pydantic)" \
  --description "pydantic-settings: settings from environment variables"
gen starlette "$(v starlette)" "$platform" starlette.requests starlette.responses starlette.status --depends "$P" \
  --description "starlette: ASGI toolkit under FastAPI"
gen fastapi "$(v fastapi)" "$platform" fastapi fastapi.responses fastapi.security fastapi.middleware.cors fastapi.testclient \
  --depends "$P,pydantic@$(v pydantic),starlette@$(v starlette)" --description "FastAPI: REST APIs"
gen sqlalchemy "$(v sqlalchemy)" "$platform" sqlalchemy sqlalchemy.orm sqlalchemy.ext.asyncio --depends "$P" \
  --description "SQLAlchemy 2: ORM (async with asyncpg, aiomysql, aiosqlite)"
gen asyncpg "$(v asyncpg)" "$platform" asyncpg --depends "$P" --description "asyncpg: PostgreSQL driver"
gen redis "$(v redis)" "$platform" redis.asyncio --roots redis --depends "$P" --description "redis-py asyncio client"
gen aiokafka "$(v aiokafka)" "$platform" aiokafka --depends "$P" --description "aiokafka: Kafka producer and consumer"
gen openai "$(v openai)" "$platform" openai --depends "$P,pydantic@$(v pydantic)" --description "OpenAI Python SDK"
gen ollama "$(v ollama)" "$platform" ollama --depends "$P,pydantic@$(v pydantic)" --description "Ollama Python client"
gen chromadb "$(v chromadb)" "$platform" chromadb --depends "$P" --description "Chroma: vector database"
LC="langchain_core@$(v langchain-core)"
gen langchain_core "$(v langchain-core)" "$platform" langchain_core.messages langchain_core.prompts langchain_core.tools \
  langchain_core.runnables langchain_core.output_parsers langchain_core.documents langchain_core.embeddings \
  langchain_core.language_models langchain_core.vectorstores langchain_core.retrievers \
  --depends "$P,pydantic@$(v pydantic)" --description "LangChain core: messages, prompts, tools, runnables"
gen langchain "$(v langchain)" "$platform" langchain.agents langchain.chat_models langchain.embeddings \
  --depends "$LC" --description "LangChain: agents and model factories"
gen langgraph "$(v langgraph)" "$platform" langgraph.graph langgraph.graph.message langgraph.prebuilt langgraph.types \
  langgraph.checkpoint.memory --depends "$LC" --description "LangGraph: state graphs, agents, checkpoints"
gen langchain_openai "$(v langchain-openai)" "$platform" langchain_openai --depends "$LC,openai@$(v openai)" \
  --description "LangChain integration of OpenAI"
gen langchain_ollama "$(v langchain-ollama)" "$platform" langchain_ollama --depends "$LC,ollama@$(v ollama)" \
  --description "LangChain integration of Ollama"
gen langchain_chroma "$(v langchain-chroma)" "$platform" langchain_chroma --depends "$LC,chromadb@$(v chromadb)" \
  --description "LangChain integration of Chroma"
# the runtime of the fastapi dialect (packages/dragonfly-py, installed in the venv with pip install -e)
gen dragonfly "$("$py" -c "import importlib.metadata as m; print(m.version('dragonfly-py'))")" "$platform" dragonfly dragonfly.ai dragonfly.events dragonfly.testing dragonfly.validators \
  --depends "$P,fastapi@$(v fastapi),sqlalchemy@$(v sqlalchemy),pydantic@$(v pydantic),langchain_core@$(v langchain-core),langgraph@$(v langgraph)" \
  --description "Dragonfly for Python: the runtime of the fastapi dialect"
