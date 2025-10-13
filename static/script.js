const API_BASE = 'http://127.0.0.1:8080/api/v1';

let batchEvals = [];
let allHistory = [];
let historyCurrentPage = 1;
const HISTORY_ITEMS_PER_PAGE = 10;
let allResults = [];

// Tab Management
document.addEventListener('DOMContentLoaded', () => {
    initTabs();
    initSingleEvalForm();
    initBatchEval();
    initHistoryTab();
    loadResults();
});

function initTabs() {
    const tabBtns = document.querySelectorAll('.tab-btn');
    const tabContents = document.querySelectorAll('.tab-content');

    tabBtns.forEach(btn => {
        btn.addEventListener('click', () => {
            const targetTab = btn.dataset.tab;

            tabBtns.forEach(b => b.classList.remove('active'));
            tabContents.forEach(c => c.classList.remove('active'));

            btn.classList.add('active');
            document.getElementById(`${targetTab}-tab`).classList.add('active');
            if (targetTab === 'history') {
                loadHistory();
            }
        });
    });
}

// Single Eval Form
function initSingleEvalForm() {
    const form = document.getElementById('single-eval-form');
    const resultContainer = document.getElementById('single-result');

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        
        const submitBtn = form.querySelector('button[type="submit"]');
        const btnText = submitBtn.querySelector('.btn-text');
        const spinner = submitBtn.querySelector('.spinner');
        
        submitBtn.disabled = true;
        btnText.style.display = 'none';
        spinner.classList.remove('hidden');
        resultContainer.classList.add('hidden');

        const formData = new FormData(form);
        const payload = {
            model: formData.get('model'),
            prompt: formData.get('prompt'),
            expected: formData.get('expected') || null,
            judge_model: formData.get('judge_model') || null,
            criteria: formData.get('criteria') || null
        };

        try {
            const response = await fetch(`${API_BASE}/evals/run`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });

            const data = await response.json();
            displaySingleResult(data);
            
            // Add to results list
            allResults.unshift({
                ...data,
                timestamp: new Date().toISOString()
            });
            updateResultsList();

        } catch (error) {
            displayError(resultContainer, `Failed to run evaluation: ${error.message}`);
        } finally {
            submitBtn.disabled = false;
            btnText.style.display = 'inline';
            spinner.classList.add('hidden');
        }
    });
}

function displaySingleResult(data) {
    const container = document.getElementById('single-result');
    container.classList.remove('hidden');
    
    if (data.error) {
        container.innerHTML = `<div class="error-message">${data.error}</div>`;
        return;
    }

    const result = data.result;
    let html = '<div class="result-header">';
    html += '<h3>Evaluation Result</h3>';
    
    if (result.judge_result) {
        const verdict = result.judge_result.verdict.toLowerCase();
        html += `<span class="verdict-badge verdict-${verdict === 'pass' ? 'pass' : verdict === 'fail' ? 'fail' : 'uncertain'}">
            ${verdict === 'pass' ? '✅ PASS' : verdict === 'fail' ? '❌ FAIL' : '⚠️ UNCERTAIN'}
        </span>`;
    }
    html += '</div>';

    html += '<div class="result-section">';
    html += '<h4>Model</h4>';
    html += `<pre>${result.model}</pre>`;
    html += '</div>';

    // Add latency information
    if (result && result.latency_ms !== undefined) {
        html += '<div class="result-section">';
        html += '<h4>⏱️ Performance Metrics</h4>';
        html += '<pre>';
        html += `Model Response Time: ${result.latency_ms}ms\n`;
        if (result.judge_latency_ms) {
            html += `Judge Response Time: ${result.judge_latency_ms}ms\n`;
        }
        html += `Total Evaluation Time: ${result.total_latency_ms}ms`;
        html += '</pre>';
        html += '</div>';
    }

    html += '<div class="result-section">';
    html += '<h4>Prompt</h4>';
    html += `<pre>${escapeHtml(result.prompt)}</pre>`;
    html += '</div>';

    html += '<div class="result-section">';
    html += '<h4>Model Output</h4>';
    html += `<pre>${escapeHtml(result.model_output)}</pre>`;
    html += '</div>';

    if (result.expected) {
        html += '<div class="result-section">';
        html += '<h4>Expected Output</h4>';
        html += `<pre>${escapeHtml(result.expected)}</pre>`;
        html += '</div>';
    }

    if (result.judge_result && result.judge_result.reasoning) {
        html += '<div class="result-section">';
        html += '<h4>Judge Reasoning</h4>';
        html += `<pre>${escapeHtml(result.judge_result.reasoning)}</pre>`;
        html += '</div>';
    }

    container.innerHTML = html;
}

// Batch Eval
function initBatchEval() {
    document.getElementById('add-eval-btn').addEventListener('click', addBatchEvalItem);
    document.getElementById('run-batch-btn').addEventListener('click', runBatchEval);
    
    // Add first eval item by default
    addBatchEvalItem();
}

function addBatchEvalItem() {
    const container = document.getElementById('batch-evals-container');
    const index = batchEvals.length;
    
    const item = document.createElement('div');
    item.className = 'batch-eval-item';
    item.dataset.index = index;
    
    item.innerHTML = `
        <h3>
            Evaluation ${index + 1}
            <button type="button" class="remove-eval-btn" onclick="removeBatchEval(${index})">Remove</button>
        </h3>
        <div class="form-group">
            <label>Model Name</label>
            <input type="text" class="batch-model" placeholder="gemini-1.5-flash" required>
        </div>
        <div class="form-group">
            <label>Prompt</label>
            <textarea class="batch-prompt" rows="3" placeholder="Enter your prompt..." required></textarea>
        </div>
        <div class="form-group">
            <label>Expected Output (Optional)</label>
            <textarea class="batch-expected" rows="2" placeholder="Expected response..."></textarea>
        </div>
        <div class="form-row">
            <div class="form-group">
                <label>Judge Model (Optional)</label>
                <input type="text" class="batch-judge" placeholder="gemini-1.5-pro">
            </div>
            <div class="form-group">
                <label>Criteria (Optional)</label>
                <input type="text" class="batch-criteria" placeholder="Custom criteria...">
            </div>
        </div>
    `;
    
    container.appendChild(item);
    batchEvals.push(item);
}

function removeBatchEval(index) {
    const container = document.getElementById('batch-evals-container');
    const item = container.querySelector(`[data-index="${index}"]`);
    if (item && batchEvals.length > 1) {
        item.remove();
        batchEvals = batchEvals.filter((_, i) => i !== index);
        updateBatchIndices();
    }
}

function updateBatchIndices() {
    const container = document.getElementById('batch-evals-container');
    const items = container.querySelectorAll('.batch-eval-item');
    items.forEach((item, index) => {
        item.dataset.index = index;
        item.querySelector('h3').firstChild.textContent = `Evaluation ${index + 1} `;
    });
}

async function runBatchEval() {
    const container = document.getElementById('batch-evals-container');
    const items = container.querySelectorAll('.batch-eval-item');
    const resultContainer = document.getElementById('batch-result');
    const runBtn = document.getElementById('run-batch-btn');
    
    const evals = [];
    let isValid = true;
    
    items.forEach(item => {
        const model = item.querySelector('.batch-model').value.trim();
        const prompt = item.querySelector('.batch-prompt').value.trim();
        
        if (!model || !prompt) {
            isValid = false;
            return;
        }
        
        evals.push({
            model,
            prompt,
            expected: item.querySelector('.batch-expected').value.trim() || null,
            judge_model: item.querySelector('.batch-judge').value.trim() || null,
            criteria: item.querySelector('.batch-criteria').value.trim() || null
        });
    });
    
    if (!isValid) {
        alert('Please fill in model and prompt for all evaluations');
        return;
    }
    
    runBtn.disabled = true;
    runBtn.classList.add('loading');
    resultContainer.classList.add('hidden');
    
    try {
        const response = await fetch(`${API_BASE}/evals/batch`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ evals })
        });
        
        const data = await response.json();
        displayBatchResults(data);
        
        // Add to results list
        data.results.forEach(result => {
            allResults.unshift({
                ...result,
                timestamp: new Date().toISOString()
            });
        });
        updateResultsList();
        
    } catch (error) {
        displayError(resultContainer, `Failed to run batch evaluation: ${error.message}`);
    } finally {
        runBtn.disabled = false;
        runBtn.classList.remove('loading');
    }
}

function displayBatchResults(data) {
    const container = document.getElementById('batch-result');
    container.classList.remove('hidden');
    
    let html = '<div class="batch-summary">';
    html += `<div class="summary-card"><h4>Total</h4><div class="value">${data.total}</div></div>`;
    html += `<div class="summary-card"><h4>Completed</h4><div class="value">${data.completed}</div></div>`;
    html += `<div class="summary-card"><h4>Passed</h4><div class="value" style="color: #28a745">${data.passed}</div></div>`;
    html += `<div class="summary-card"><h4>Failed</h4><div class="value" style="color: #dc3545">${data.failed}</div></div>`;
    html += '</div>';
    
    html += '<div class="batch-results-list">';
    data.results.forEach((result, index) => {
        const hasError = result.error || !result.result;
        html += `<div class="batch-result-item ${hasError ? 'error' : ''}">`;
        html += `<div class="result-header">`;
        html += `<h3>Evaluation ${index + 1}</h3>`;
        
        if (result.result && result.result.judge_result) {
            const verdict = result.result.judge_result.verdict.toLowerCase();
            html += `<span class="verdict-badge verdict-${verdict === 'pass' ? 'pass' : verdict === 'fail' ? 'fail' : 'uncertain'}">
                ${verdict === 'pass' ? '✅ PASS' : verdict === 'fail' ? '❌ FAIL' : '⚠️ UNCERTAIN'}
            </span>`;
        } else if (hasError) {
            html += `<span class="verdict-badge verdict-fail">❌ ERROR</span>`;
        }
        
        html += '</div>';
        
        if (hasError) {
            html += `<div class="error-message">${result.error || 'Unknown error'}</div>`;
        } else {
            html += `<div class="result-section"><h4>Model</h4><pre>${result.result.model}</pre></div>`;
            html += `<div class="result-section"><h4>Output</h4><pre>${escapeHtml(result.result.model_output.substring(0, 200))}${result.result.model_output.length > 200 ? '...' : ''}</pre></div>`;
        }
        
        html += '</div>';
    });
    html += '</div>';
    
    container.innerHTML = html;
}

// Results List
function updateResultsList() {
    const container = document.getElementById('results-list');
    
    if (allResults.length === 0) {
        container.innerHTML = '<p class="empty-state">No results yet. Run an evaluation to see results here.</p>';
        return;
    }
    
    let html = '';
    allResults.slice(0, 20).forEach((result, index) => {
        // Handle both live API response structure and the structure from the database/history
        const evalData = result.result?.Success ? result.result.Success : result.result;

        html += `<div class="result-item" onclick="showResultDetail(${index})">`;
        html += '<div class="result-item-header">';
        html += `<h4>${evalData ? evalData.model : 'Evaluation'}</h4>`;
        
        if (evalData && evalData.judge_result) {
            const verdict = evalData.judge_result.verdict.toLowerCase();
            html += `<span class="verdict-badge verdict-${verdict === 'pass' ? 'pass' : verdict === 'fail' ? 'fail' : 'uncertain'}">
                ${verdict.toUpperCase()}
            </span>`;
        } else if (result.error) {
            html += `<span class="verdict-badge verdict-fail">ERROR</span>`;
        }
        html += '</div>';
        
        if (evalData) {
            html += `<div class="result-item-content">`;
            html += `<strong>Prompt:</strong> ${escapeHtml(evalData.prompt.substring(0, 100))}${evalData.prompt.length > 100 ? '...' : ''}`;
            html += `</div>`;
        }
        
        html += `<div class="timestamp">${formatTimestamp(result.timestamp)}</div>`;
        html += '</div>';
    });
    
    container.innerHTML = html;
}

function showResultDetail(index) {
    const result = allResults[index];
    // The data from 'allResults' might have the nested 'Success' property.
    // We create a consistent object for displaySingleResult.
    const displayData = result.result?.Success ? { ...result, result: result.result.Success } : result;

    document.querySelector('.tab-btn[data-tab="single"]').click(); // Switch to single eval tab
    displaySingleResult(displayData);
}

function initHistoryTab() {
    // The logic is triggered by the tab click handler in initTabs
}

async function loadHistory() {
    const container = document.getElementById('history-list');
    container.innerHTML = '<p class="empty-state">Loading history...</p>';

    try {
        const response = await fetch(`${API_BASE}/evals/history`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data = await response.json();
        allHistory = data.results || [];
        historyCurrentPage = 1;
        updateHistoryList();
        updateHistoryPagination();
    } catch (error) {
        displayError(container, `Failed to load history: ${error.message}`);
    }
}

function updateHistoryList() {
    const container = document.getElementById('history-list');

    if (allHistory.length === 0) {
        container.innerHTML = '<p class="empty-state">No history found.</p>';
        return;
    }

    let html = '';
    const startIndex = (historyCurrentPage - 1) * HISTORY_ITEMS_PER_PAGE;
    const endIndex = startIndex + HISTORY_ITEMS_PER_PAGE;
    const pageItems = allHistory.slice(startIndex, endIndex);

    pageItems.forEach((result, index) => {
        const globalIndex = startIndex + index;
        html += `<div class="result-item" onclick="showHistoryDetail(${globalIndex})">`;
        html += '<div class="result-item-header">';
        html += `<h4>${result.model || 'Evaluation'}</h4>`;

        if (result.judge_verdict) {
            const verdict = result.judge_verdict.toLowerCase();
            html += `<span class="verdict-badge verdict-${verdict === 'pass' ? 'pass' : verdict === 'fail' ? 'fail' : 'uncertain'}">
                ${verdict.toUpperCase()}
            </span>`;
        } else if (result.error_message) {
            html += `<span class="verdict-badge verdict-fail">ERROR</span>`;
        }
        html += '</div>';

        if (result.prompt) {
            html += `<div class="result-item-content">`;
            html += `<strong>Prompt:</strong> ${escapeHtml(result.prompt.substring(0, 100))}${result.prompt.length > 100 ? '...' : ''}`;
            html += `</div>`;
        }

        html += `<div class="timestamp">${formatTimestamp(result.created_at)}</div>`;
        html += '</div>';
    });

    container.innerHTML = html;
}

function updateHistoryPagination() {
    const paginationContainer = document.getElementById('history-pagination');
    const pageCount = Math.ceil(allHistory.length / HISTORY_ITEMS_PER_PAGE);

    if (pageCount <= 1) {
        paginationContainer.innerHTML = '';
        return;
    }

    let html = '';
    for (let i = 1; i <= pageCount; i++) {
        html += `<button class="page-btn ${i === historyCurrentPage ? 'active' : ''}" onclick="goToHistoryPage(${i})">${i}</button>`;
    }
    paginationContainer.innerHTML = html;
}

function goToHistoryPage(pageNumber) {
    historyCurrentPage = pageNumber;
    updateHistoryList();
    updateHistoryPagination();
}

function showHistoryDetail(index) {
    if (index < 0 || index >= allHistory.length) return;
    const result = allHistory[index];
    // Switch to single eval tab and display result
    document.querySelector('.tab-btn[data-tab="single"]').click();
    displaySingleResult(transformHistoryEntryToEvalResponse(result));
}

function transformHistoryEntryToEvalResponse(historyEntry) {
    // This function transforms the flat HistoryEntry from the DB 
    // into the nested structure that displaySingleResult expects.
    if (historyEntry.error_message) {
        return {
            id: historyEntry.id,
            status: 'error',
            error: historyEntry.error_message,
        };
    }
    return {
        id: historyEntry.id,
        status: historyEntry.status,
        timestamp: historyEntry.created_at, // Pass the timestamp through
        result: {
            model: historyEntry.model,
            prompt: historyEntry.prompt,
            model_output: historyEntry.model_output,
            expected: historyEntry.expected,
            judge_result: historyEntry.judge_verdict ? { verdict: historyEntry.judge_verdict, reasoning: historyEntry.judge_reasoning } : null,
            // Note: Latency and other fields are not in history, so they will be undefined.
        }
    };
}

function loadResults() {
    const stored = localStorage.getItem('evalResults');
    if (stored) {
        allResults = JSON.parse(stored);
        updateResultsList();
    }
}

// Save results when page unloads
window.addEventListener('beforeunload', () => {
    localStorage.setItem('evalResults', JSON.stringify(allResults.slice(0, 50)));
});

// Utility Functions
function displayError(container, message) {
    container.classList.remove('hidden');
    container.innerHTML = `<div class="error-message">${escapeHtml(message)}</div>`;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatTimestamp(timestamp) {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now - date;
    
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    if (hours < 24) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    if (days < 7) return `${days} day${days > 1 ? 's' : ''} ago`;
    
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}