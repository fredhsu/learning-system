# Quick Start Guide

## Running the Application

1. **Basic setup:**
```bash
./run.sh
```

2. **With LLM integration (OpenAI):**
```bash
export LLM_API_KEY="your-openai-api-key"
./run.sh
```

3. **With local LLM (Ollama):**
```bash
export LLM_API_KEY="ollama"
export LLM_BASE_URL="http://localhost:11434/v1"
./run.sh
```

4. **Open your browser to:** http://localhost:3000

## Using the System

### Creating Knowledge Cards
1. Go to the "Cards" tab
2. Click "Create New Card" 
3. Enter your content (LaTeX math supported with `$...$` or `$$...$$`)
4. Add topics (comma-separated)
5. Save the card

### Review Sessions
1. Go to the "Review" tab
2. Answer the generated questions
3. Rate your performance: Again (1), Hard (2), Good (3), Easy (4)
4. The FSRS algorithm automatically schedules your next review

### Managing Topics
1. Go to the "Topics" tab
2. Create topics to organize your cards
3. Cards can belong to multiple topics

## API Examples

### Create a card:
```bash
curl -X POST http://localhost:3000/api/cards \
  -H "Content-Type: application/json" \
  -d '{
    "content": "The quadratic formula is $x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$",
    "topic_ids": [],
    "links": null
  }'
```

### Get cards due for review:
```bash
curl http://localhost:3000/api/cards/due
```

### Create a topic:
```bash
curl -X POST http://localhost:3000/api/topics \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Mathematics",
    "description": "Mathematical concepts and formulas"
  }'
```

## Example Card Content

**Mathematics:**
```
The derivative of $e^x$ is $e^x$.

Key properties:
- $\\frac{d}{dx}[e^x] = e^x$
- $\\int e^x dx = e^x + C$
- $e^0 = 1$
```

**Programming:**
```
# Python List Comprehension

Syntax: `[expression for item in iterable if condition]`

Example: `squares = [x**2 for x in range(10) if x % 2 == 0]`

This creates a list of squares of even numbers from 0 to 9.
```

**History:**
```
The French Revolution began in 1789 and lasted until 1799.

Key events:
- Storming of the Bastille (July 14, 1789)
- Declaration of the Rights of Man (August 1789) 
- Reign of Terror (1793-1794)
- Napoleon's rise to power (1799)
```

## Troubleshooting

- **Database locked:** Make sure only one instance is running
- **LLM errors:** Check your API key and network connection
- **Port conflicts:** Change the PORT environment variable
- **Build errors:** Run `cargo clean` and try again