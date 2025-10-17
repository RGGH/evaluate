const API_BASE = 'http://127.0.0.1:8080/api/v1';

let batchEvals = [];
let allHistory = [];
const HISTORY_ITEMS_PER_PAGE = 10;
let historyCurrentPage = 1;

// State for new tabs
let summaryInitialized = false;
let resultsInitialized = false;
let resultsWs;
let resultsCharts = {};
let resultsHistoryData = [];

// Tab Management
document.addEventListener('DOMContentLoaded', () => {
    initTabs();
    initSingleEvalForm();
    initBatchEval();
    loadAndPopulateModels(); // This will now fetch models and populate dropdowns
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
            const activeTabContent = document.getElementById(`${targetTab}-tab`);
            if (activeTabContent) {
                activeTabContent.classList.add('active');
            }

            // Load content for tabs on demand
            switch (targetTab) {
                case 'history':
                    loadHistory();
                    break;
                case 'summary':
                    initSummaryTab();
                    break;
                case 'results':
                    initResultsTab();
                    break;
            }
        });
    });
}

// Enhanced error detection and display
function isApiKeyError(errorMessage) {
    if (!errorMessage) return false;
    const errorLower = errorMessage.toLowerCase();
    return errorLower.includes('failed to respond') ||
           errorLower.includes('modelfailure') ||
           errorLower.includes('401') ||
           errorLower.includes('403') ||
           errorLower.includes('permission_denied') ||
           errorLower.includes('api key') ||
           errorLower.includes('authentication');
}

function displayError(container, message, modelName = null) {
    container.classList.remove('hidden');
    
    const isKeyError = isApiKeyError(message);
    
    let errorHtml = '<div class="error-message">';
    errorHtml += '<span style="font-size: 1.2em;">⚠️</span> ';
    
    if (isKeyError && modelName) {
        errorHtml += `Model '${escapeHtml(modelName)}' failed to respond`;
    } else {
        errorHtml += escapeHtml(message);
    }
    
    errorHtml += '</div>';
    
    if (isKeyError) {
        errorHtml += `
            <div style="background: #fff3cd; border: 1px solid #ffc107; border-radius: 8px; padding: 16px; margin-top: 16px;">
                <div style="font-weight: 600; color: #856404; margin-bottom: 8px;">
                    💡 Possible causes:
                </div>
                <ul style="margin: 8px 0; padding-left: 24px; color: #856404;">
                    <li>Invalid or missing API key in <code>src/config.toml</code></li>
                    <li>API key doesn't have access to the requested model</li>
                    <li>API quota exceeded or billing not enabled</li>
                    <li>Network connectivity issues</li>
                </ul>
                <div style="font-weight: 600; color: #856404; margin-top: 12px; margin-bottom: 8px;">
                    🔧 How to fix:
                </div>
                <ol style="margin: 8px 0; padding-left: 24px; color: #856404;">
                    <li>Check your <code>src/config.toml</code> file</li>
                    <li>Verify your API key is correct and active</li>
                    <li>Ensure the model name is spelled correctly (e.g., "gemini-2.5-flash")</li>
                    <li>Check your API quota and billing status at <a href="https://aistudio.google.com" target="_blank">Google AI Studio</a></li>
                </ol>
            </div>
        `;
    }
    
    container.innerHTML = errorHtml;
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
            
            if (data.error) {
                displayError(resultContainer, data.error, payload.model);
            } else {
                displaySingleResult(data);
            }

        } catch (error) {
            displayError(resultContainer, `Failed to run evaluation: ${error.message}`, payload.model);
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
        displayError(container, data.error);
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

    // Add token usage information
    if (result.token_usage || result.judge_token_usage) {
        html += '<div class="result-section">';
        html += '<h4>🪙 Token Usage</h4>';
        html += '<pre>';
        if (result.token_usage) {
            html += `Model Tokens: ${result.token_usage.input_tokens || 'N/A'} (prompt) / ${result.token_usage.output_tokens || 'N/A'} (completion)\n`;
        }
        if (result.judge_token_usage) {
            html += `Judge Tokens: ${result.judge_token_usage.input_tokens || 'N/A'} (prompt) / ${result.judge_token_usage.output_tokens || 'N/A'} (completion)`;
        }
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
            <select class="batch-model" required></select>
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
                <select class="batch-judge"></select>
            </div>
            <div class="form-group">
                <label>Criteria (Optional)</label>
                <input type="text" class="batch-criteria" placeholder="Custom criteria...">
            </div>
        </div>
    `;
    
    container.appendChild(item);
    batchEvals.push(item);

    // Populate the new dropdowns with the already-loaded models
    populateModelDropdown(item.querySelector('.batch-model'));
    populateModelDropdown(item.querySelector('.batch-judge'), true);
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
        displayBatchResults(data, evals);
        
    } catch (error) {
        displayError(resultContainer, `Failed to run batch evaluation: ${error.message}`);
    } finally {
        runBtn.disabled = false;
        runBtn.classList.remove('loading');
    }
}

function displayBatchResults(data, originalEvals) {
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
        const modelName = originalEvals[index]?.model || result.result?.model || 'Unknown';
        
        html += `<div class="batch-result-item ${hasError ? 'error' : ''}">`;
        html += `<div class="result-header">`;
        html += `<h3>Evaluation ${index + 1} - ${escapeHtml(modelName)}</h3>`;
        
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
            const errorMsg = result.error || 'Unknown error';
            html += `<div class="error-message">⚠️ ${escapeHtml(errorMsg)}</div>`;
            
            if (isApiKeyError(errorMsg)) {
                html += `
                    <div style="background: #fff3cd; border: 1px solid #ffc107; border-radius: 8px; padding: 12px; margin-top: 12px; font-size: 0.9em;">
                        <strong style="color: #856404;">💡 Check API key configuration</strong><br>
                        <span style="color: #856404;">Verify your <code>src/config.toml</code> has a valid API key for this model.</span>
                    </div>
                `;
            }
        } else {
            html += `<div class="result-section"><h4>Model</h4><pre>${result.result.model}</pre></div>`;
            html += `<div class="result-section"><h4>Output</h4><pre>${escapeHtml(result.result.model_output.substring(0, 200))}${result.result.model_output.length > 200 ? '...' : ''}</pre></div>`;
        }
        
        html += '</div>';
    });
    html += '</div>';
    
    container.innerHTML = html;
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
        } else if (result.error_message) {
            html += `<div class="result-item-content" style="color: #dc3545;">`;
            html += `<strong>Error:</strong> ${escapeHtml(result.error_message.substring(0, 100))}`;
            if (isApiKeyError(result.error_message)) {
                html += ` <small>(Check API key)</small>`;
            }
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
        timestamp: historyEntry.created_at,
        result: {
            model: historyEntry.model,
            prompt: historyEntry.prompt,
            model_output: historyEntry.model_output,
            expected: historyEntry.expected,
            judge_result: historyEntry.judge_verdict ? { 
                verdict: historyEntry.judge_verdict, 
                reasoning: historyEntry.judge_reasoning 
            } : null,
            latency_ms: historyEntry.latency_ms,
            judge_latency_ms: historyEntry.judge_latency_ms,
            token_usage: (historyEntry.input_tokens !== null || historyEntry.output_tokens !== null) ? { input_tokens: historyEntry.input_tokens, output_tokens: historyEntry.output_tokens } : null,
            judge_token_usage: (historyEntry.judge_input_tokens !== null || historyEntry.judge_output_tokens !== null) ? { input_tokens: historyEntry.judge_input_tokens, output_tokens: historyEntry.judge_output_tokens } : null,
        }
    };
}

async function loadAndPopulateModels() {
    try {
        const response = await fetch(`${API_BASE}/models`);
        if (!response.ok) {
            throw new Error('Failed to fetch models');
        }
        const data = await response.json();
        const models = data.models || [];

        // Store models globally so other functions can access them
        window.availableModels = models;

        // Populate all model dropdowns on the page, including batch items
        document.querySelectorAll('#model, .batch-model').forEach(select => {
            populateModelDropdown(select);
        });
        document.querySelectorAll('#judge-model, .batch-judge').forEach(select => {
            populateModelDropdown(select, true);
        });

    } catch (error) {
        console.error('Error loading models:', error);
        // You could display an error to the user here
    }
}

function populateModelDropdown(selectElement, isOptional = false) {
    if (!selectElement || !window.availableModels) return;

    selectElement.innerHTML = isOptional ? '<option value="">-- Select Model --</option>' : '';

    window.availableModels.forEach(model => {
        selectElement.innerHTML += `<option value="${model}">${model}</option>`;
    });
}

// Utility Functions
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

// --- Summary Tab Logic ---
function initSummaryTab() {
    if (summaryInitialized) {
        loadSummaryResults(); // Refresh data if already initialized
        return;
    }
    summaryInitialized = true;
    loadSummaryResults();
    setInterval(loadSummaryResults, 10000); // Auto-refresh
}

async function loadSummaryResults() {
    try {
        const response = await fetch(`${API_BASE}/evals/history`);
        const data = await response.json();
        
        if (data.results && data.results.length > 0) {
            updateSummaryMetrics(data.results);
            displaySummaryResultsTable(data.results);
        } else {
            showSummaryEmptyState();
        }
    } catch (error) {
        console.error('Failed to load summary results:', error);
        document.getElementById('summaryResultsTable').innerHTML = 
            '<div class="empty-state"><div class="empty-state-icon">❌</div><p>Failed to load results</p></div>';
    }
}

function updateSummaryMetrics(results) {
    const total = results.length;
    const passed = results.filter(r => r.judge_verdict === 'Pass').length;
    const passRate = total > 0 ? Math.round((passed / total) * 100) : 0;
    const uniqueModels = new Set(results.map(r => r.model).filter(Boolean));    
    const latencies = results.map(r => (r.latency_ms || 0) + (r.judge_latency_ms || 0)).filter(l => l > 0);
    const avgLatency = latencies.length > 0 ? Math.round(latencies.reduce((a, b) => a + b, 0) / latencies.length) : 0;
    const totalTokens = results.reduce((acc, r) => (r.input_tokens || 0) + (r.output_tokens || 0) + (r.judge_input_tokens || 0) + (r.judge_output_tokens || 0) + acc, 0);
    
    document.getElementById('summaryTotalCount').textContent = total;
    document.getElementById('summaryPassRate').textContent = `${passRate}%`;
    document.getElementById('summaryAvgLatency').textContent = avgLatency > 0 ? `${avgLatency}ms` : 'N/A';
    document.getElementById('summaryModelCount').textContent = uniqueModels.size;
    document.getElementById('summaryTotalTokens').textContent = totalTokens.toLocaleString();
}

function displaySummaryResultsTable(results) {
    const getStatusClass = (verdict) => {
        if (!verdict) return 'error';
        const lower = verdict.toLowerCase();
        return lower === 'pass' ? 'pass' : (lower === 'fail' ? 'fail' : 'error');
    };
    const truncate = (str, length) => str && str.length > length ? str.substring(0, length) + '...' : str || 'N/A';

    const tableHTML = `
        <table> 
            <thead><tr><th>Model</th><th>Prompt</th><th>Output</th><th>Verdict</th><th>Judge</th><th>Tokens (M P/C)</th><th>Tokens (J P/C)</th><th>Timestamp</th></tr></thead>
            <tbody>
                ${results.slice(0, 50).map(result => `
                    <tr>
                        <td><strong>${result.model || 'N/A'}</strong></td>
                        <td class="prompt-cell" title="${escapeHtml(result.prompt || '')}">${truncate(result.prompt, 60)}</td>
                        <td class="output-cell" title="${escapeHtml(result.model_output || '')}">${truncate(result.model_output, 50)}</td>
                        <td class="status-${getStatusClass(result.judge_verdict)}">${result.judge_verdict || 'N/A'}</td>
                        <td>${result.judge_model || 'N/A'}</td>
                        <td>${result.input_tokens || 'N/A'} / ${result.output_tokens || 'N/A'}</td>
                        <td>${result.judge_input_tokens || 'N/A'} / ${result.judge_output_tokens || 'N/A'}</td>
                        <td>${new Date(result.created_at).toLocaleString()}</td>
                    </tr>
                `).join('')}
            </tbody>
        </table>`;
    document.getElementById('summaryResultsTable').innerHTML = tableHTML;
}

function showSummaryEmptyState() {
    document.getElementById('summaryResultsTable').innerHTML = `
        <div class="empty-state">
            <div class="empty-state-icon">📊</div>
            <p>No evaluation results yet. Run some evaluations to see results here!</p>
        </div>`;
}

// --- Results Tab (Live Dashboard) Logic ---
function initResultsTab() {
    if (resultsInitialized) return;
    resultsInitialized = true;
    
    connectWebSocket();
    fetchResultsHistory();
    setInterval(fetchResultsHistory, 30000);
}

function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    resultsWs = new WebSocket(`${protocol}//${window.location.host}/api/v1/ws`);
    
    resultsWs.onopen = () => console.log('Results WebSocket connected');
    resultsWs.onmessage = (e) => {
        console.log('Real-time update:', JSON.parse(e.data));
        fetchResultsHistory();
    };
    resultsWs.onerror = (e) => console.error('Results WebSocket error:', e);
    resultsWs.onclose = () => {
        console.log('Results WebSocket closed, reconnecting...');
        setTimeout(connectWebSocket, 3000);
    };
}

async function fetchResultsHistory() {
    try {
        const res = await fetch('/api/v1/evals/history');
        const data = await res.json();
        resultsHistoryData = data.results || [];
        updateResultsDashboard();
    } catch (e) {
        console.error('Failed to fetch results history:', e);
    }
}

function updateResultsDashboard() {
    updateResultsStats();
    updateResultsCharts();
    updateResultsTable();
}

function updateResultsStats() {
    const total = resultsHistoryData.length;
    const passed = resultsHistoryData.filter(e => e.judge_verdict === 'Pass').length;
    const passRate = total > 0 ? Math.round((passed / total) * 100) : 0;
    const latencies = resultsHistoryData.map(e => (e.latency_ms || 0) + (e.judge_latency_ms || 0)).filter(l => l > 0);
    const avgLatency = latencies.length > 0 ? Math.round(latencies.reduce((a, b) => a + b, 0) / latencies.length) : 0;
    const models = new Set(resultsHistoryData.map(e => e.model).filter(Boolean));
    const totalTokens = resultsHistoryData.reduce((acc, e) => (e.input_tokens || 0) + (e.output_tokens || 0) + (e.judge_input_tokens || 0) + (e.judge_output_tokens || 0) + acc, 0);
    
    document.getElementById('totalEvals').textContent = total;
    document.getElementById('passRate').textContent = passRate + '%';
    document.getElementById('avgLatency').textContent = avgLatency + 'ms';
    document.getElementById('activeModels').textContent = models.size;
    document.getElementById('totalTokens').textContent = totalTokens.toLocaleString();
}

function updateResultsCharts() {
    const createOrUpdateChart = (chartId, type, data, options) => {
        const ctx = document.getElementById(chartId)?.getContext('2d');
        if (!ctx) return;
        if (resultsCharts[chartId]) resultsCharts[chartId].destroy();
        resultsCharts[chartId] = new Chart(ctx, { type, data, options });
    };

    // Verdict Chart
    const verdicts = resultsHistoryData.reduce((acc, e) => {
        const v = e.judge_verdict || 'No Judge';
        acc[v] = (acc[v] || 0) + 1;
        return acc;
    }, {});
    createOrUpdateChart('verdictChart', 'doughnut', {
        labels: Object.keys(verdicts).map(label => label === 'No Judge' ? 'Error/No Judge' : label),
        datasets: [{ data: Object.values(verdicts), backgroundColor: ['#28a745', '#dc3545', '#ffc107', '#6c757d'] }]
    }, { responsive: true, plugins: { legend: { position: 'bottom' } } });

    // Model Chart
    const models = resultsHistoryData.reduce((acc, e) => {
        if (!e.model) return acc;
        if (!acc[e.model]) acc[e.model] = { pass: 0, fail: 0 };
        if (e.judge_verdict === 'Pass') acc[e.model].pass++;
        else if (e.judge_verdict === 'Fail') acc[e.model].fail++;
        return acc;
    }, {});
    createOrUpdateChart('modelChart', 'bar', {
        labels: Object.keys(models),
        datasets: [
            { label: 'Pass', data: Object.values(models).map(m => m.pass), backgroundColor: '#48bb78' },
            { label: 'Fail', data: Object.values(models).map(m => m.fail), backgroundColor: '#f56565' }
        ]
    }, { responsive: true, scales: { y: { beginAtZero: true, stacked: true }, x: { stacked: true } } });

    // Latency Chart
    const recent = resultsHistoryData.slice(-20);
    createOrUpdateChart('latencyChart', 'line', {
        labels: recent.map((_, i) => i + 1),
        datasets: [{
            label: 'Latency (ms)', data: recent.map(e => (e.latency_ms || 0) + (e.judge_latency_ms || 0)),
            borderColor: '#ffc107', backgroundColor: 'rgba(255, 193, 7, 0.1)', tension: 0.4, fill: true
        }]
    }, { responsive: true, scales: { y: { beginAtZero: true } } });

    // Timeline Chart
    const byDate = resultsHistoryData.reduce((acc, e) => {
        const date = new Date(e.created_at).toLocaleDateString();
        acc[date] = (acc[date] || 0) + 1;
        return acc;
    }, {});
    createOrUpdateChart('timelineChart', 'bar', {
        labels: Object.keys(byDate),
        datasets: [{ label: 'Evaluations', data: Object.values(byDate), backgroundColor: 'rgba(192, 192, 192, 0.7)' }]
    }, { responsive: true, scales: { y: { beginAtZero: true } } });

    // Token Chart
    const tokensByModel = resultsHistoryData.reduce((acc, e) => {
        if (!e.model) return acc;
        if (!acc[e.model]) acc[e.model] = { prompt: 0, completion: 0, judge_prompt: 0, judge_completion: 0 };
        acc[e.model].prompt += e.input_tokens || 0;
        acc[e.model].completion += e.output_tokens || 0;
        acc[e.model].judge_prompt += e.judge_input_tokens || 0;
        acc[e.model].judge_completion += e.judge_output_tokens || 0;
        return acc;
    }, {});
    createOrUpdateChart('tokenChart', 'bar', {
        labels: Object.keys(tokensByModel),
        datasets: [
            { label: 'Model Prompt Tokens', data: Object.values(tokensByModel).map(m => m.prompt), backgroundColor: '#4299e1' },
            { label: 'Model Completion Tokens', data: Object.values(tokensByModel).map(m => m.completion), backgroundColor: '#ed8936' },
            { label: 'Judge Prompt Tokens', data: Object.values(tokensByModel).map(m => m.judge_prompt), backgroundColor: '#667eea' },
            { label: 'Judge Completion Tokens', data: Object.values(tokensByModel).map(m => m.judge_completion), backgroundColor: '#a0aec0' }
        ]
    }, { responsive: true, scales: { x: { stacked: true }, y: { stacked: true, beginAtZero: true } } });
}

function updateResultsTable() {
    const tbody = document.getElementById('resultsBody');
    if (!tbody) return;
    const recentData = resultsHistoryData.slice(-50).reverse();
    tbody.innerHTML = recentData.map(e => {
        const verdict = e.judge_verdict || 'ERROR';
        const badgeClass = `verdict-${(verdict).toLowerCase()}`;
        const latency = (e.latency_ms || 0) + (e.judge_latency_ms || 0);
        return `
            <tr>
                <td>${new Date(e.created_at).toLocaleString()}</td>
                <td>${e.model || 'N/A'}</td>
                <td title="${escapeHtml(e.prompt || '')}">${(e.prompt || '').substring(0, 50)}...</td>
                <td><span class="verdict-badge ${badgeClass}">${verdict.toUpperCase()}</span></td>
                <td>${latency > 0 ? latency + 'ms' : 'N/A'}</td>
            </tr>
        `;
    }).join('');
}