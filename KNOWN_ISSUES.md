# Known Issues

## OpenAI API Quota Exceeded (ACTIVE)

**Status:** Active  
**Severity:** Medium  
**Component:** LLM Service / Quiz Generation  
**Error Code:** 429 Too Many Requests  

### Description
The OpenAI API is returning a "429 Too Many Requests" error with an "insufficient_quota" response, indicating that the current API key has exceeded its usage quota.

### Symptoms
- Quiz generation shows "Error generating quiz: error decoding response body"
- Answer grading shows "Error grading answer: error decoding response body"
- System falls back to local quiz generation (basic questions)
- Server logs show: `API Error (429 Too Many Requests): insufficient_quota`

### Root Cause
The OpenAI API key configured in `.env` has exceeded its usage quota or the associated account needs billing setup.

### Workarounds
The system is designed with graceful fallback:
- ✅ Local quiz generation continues to work
- ✅ Basic questions are generated for all cards
- ✅ Core learning system functionality remains intact

### Resolution Options

#### Option 1: Update API Key
1. Obtain a new OpenAI API key with available quota
2. Update `LLM_API_KEY` in `.env` file
3. Restart the application

#### Option 2: Set Up Billing
1. Log into your OpenAI account
2. Navigate to billing settings
3. Add payment method and increase quota
4. Wait for quota reset

#### Option 3: Use Local LLM (Recommended)
1. Install Ollama: `curl -fsSL https://ollama.ai/install.sh | sh`
2. Pull a model: `ollama pull llama2`
3. Update `.env` file:
   ```
   LLM_API_KEY=ollama
   LLM_BASE_URL=http://localhost:11434/v1
   ```
4. Restart the application

### Technical Details
- **First Observed:** 2025-08-14
- **API Endpoint:** `https://api.openai.com/v1/chat/completions`
- **Model Used:** `gpt-3.5-turbo`
- **Fallback Implementation:** Local quiz generation in `generate_quiz_questions_local()`

### Related Files
- `src/llm_service.rs` - LLM integration and error handling
- `.env` - API configuration
- `src/api.rs` - Quiz endpoint implementation

### Monitoring
To check if the issue is resolved:
```bash
curl -s "http://localhost:3000/api/cards/[CARD_ID]/quiz"
```

Look for:
- ✅ **Resolved:** Multiple diverse questions with proper JSON structure
- ❌ **Still broken:** Single fallback question about "main concept"

---

*Last Updated: 2025-08-14*  
*Commit: 311b73f*