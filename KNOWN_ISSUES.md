# Known Issues

*No active issues at this time.*

## Recently Resolved Issues

### OpenAI API Quota Exceeded (RESOLVED - 2025-08-19)

**Status:** Resolved  
**Severity:** Medium  
**Component:** LLM Service / Quiz Generation  
**Error Code:** 429 Too Many Requests  

### Description
The OpenAI API was returning a "429 Too Many Requests" error with an "insufficient_quota" response, indicating that the current API key had exceeded its usage quota.

### Resolution
The OpenAI API quota issue has been resolved. Testing confirms that:
- ✅ Quiz generation is working with proper OpenAI API responses
- ✅ LLM service successfully generates diverse, structured questions
- ✅ No API quota errors observed during testing

### Verification
Confirmed working on 2025-08-19 with successful quiz generation returning properly formatted JSON responses with multiple question types (multiple_choice, short_answer, problem_solving).

---

*Last Updated: 2025-08-19*  
*Commit: b3b445c*