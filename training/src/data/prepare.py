"""Corpus preparation utilities and seed generation for LDrago training domains."""

from __future__ import annotations

import os
from typing import List


def generate_seed_coding_corpus() -> List[str]:
    """Generate a rich collection of seed coding snippets.

    Includes:
    - Sorting algorithms (QuickSort, MergeSort, HeapSort, RadixSort, Binary Search)
    - Tree and graph traversal (BST, BFS/DFS, Trie, LCA, Topological Sort)
    - REST API handlers (FastAPI CRUD, JWT Auth Middleware, Cursor Pagination, Axum handler)
    - Async utilities (Worker Pool, Exponential Backoff, Rate Limiter, Event Bus, Batcher)
    """
    corpus: List[str] = [
        # 1. QuickSort with 3-way partitioning
        """<CODE>
def quicksort_3way(arr: list[int]) -> list[int]:
    \"\"\"Three-way QuickSort (Dutch National Flag partitioning).\"\"\"
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    less = [x for x in arr if x < pivot]
    equal = [x for x in arr if x == pivot]
    greater = [x for x in arr if x > pivot]
    return quicksort_3way(less) + equal + quicksort_3way(greater)
</CODE>""",
        # 2. MergeSort
        """<CODE>
def mergesort(arr: list[int]) -> list[int]:
    \"\"\"Divide-and-conquer MergeSort algorithm.\"\"\"
    if len(arr) <= 1:
        return arr
    mid = len(arr) // 2
    left = mergesort(arr[:mid])
    right = mergesort(arr[mid:])
    merged = []
    i = j = 0
    while i < len(left) and j < len(right):
        if left[i] <= right[j]:
            merged.append(left[i])
            i += 1
        else:
            merged.append(right[j])
            j += 1
    merged.extend(left[i:])
    merged.extend(right[j:])
    return merged
</CODE>""",
        # 3. Binary Search
        """<CODE>
def binary_search(arr: list[int], target: int) -> int:
    \"\"\"Find exact index of target in sorted array, or return -1.\"\"\"
    low, high = 0, len(arr) - 1
    while low <= high:
        mid = (low + high) // 2
        if arr[mid] == target:
            return mid
        elif arr[mid] < target:
            low = mid + 1
        else:
            high = mid - 1
    return -1

def lower_bound(arr: list[int], target: int) -> int:
    \"\"\"Return first index where arr[i] >= target.\"\"\"
    low, high = 0, len(arr)
    while low < high:
        mid = (low + high) // 2
        if arr[mid] < target:
            low = mid + 1
        else:
            high = mid
    return low
</CODE>""",
        # 4. Heap / Priority Queue
        """<CODE>
import heapq

class PriorityQueue:
    \"\"\"Thread-safe priority queue wrapper around heapq.\"\"\"
    def __init__(self) -> None:
        self._queue: list[tuple[int, int, object]] = []
        self._index: int = 0

    def push(self, item: object, priority: int) -> None:
        heapq.heappush(self._queue, (priority, self._index, item))
        self._index += 1

    def pop(self) -> object:
        if not self._queue:
            raise IndexError("pop from an empty priority queue")
        return heapq.heappop(self._queue)[-1]

    def __len__(self) -> int:
        return len(self._queue)
</CODE>""",
        # 5. Radix Sort
        """<CODE>
def radix_sort(arr: list[int]) -> list[int]:
    \"\"\"LSD Radix Sort for non-negative integers.\"\"\"
    if not arr:
        return arr
    max_val = max(arr)
    exp = 1
    res = list(arr)
    while max_val // exp > 0:
        buckets = [[] for _ in range(10)]
        for num in res:
            digit = (num // exp) % 10
            buckets[digit].append(num)
        res = [item for bucket in buckets for item in bucket]
        exp *= 10
    return res
</CODE>""",
        # 6. Binary Search Tree Traversal
        """<CODE>
class TreeNode:
    def __init__(self, val: int = 0, left: TreeNode | None = None, right: TreeNode | None = None):
        self.val = val
        self.left = left
        self.right = right

def inorder_traversal(root: TreeNode | None) -> list[int]:
    result, stack, curr = [], [], root
    while curr or stack:
        while curr:
            stack.append(curr)
            curr = curr.left
        curr = stack.pop()
        result.append(curr.val)
        curr = curr.right
    return result
</CODE>""",
        # 7. Level-order BFS Traversal
        """<CODE>
from collections import deque

def level_order_traversal(root: TreeNode | None) -> list[list[int]]:
    if not root:
        return []
    result: list[list[int]] = []
    queue: deque[TreeNode] = deque([root])
    while queue:
        level_size = len(queue)
        current_level: list[int] = []
        for _ in range(level_size):
            node = queue.popleft()
            current_level.append(node.val)
            if node.left:
                queue.append(node.left)
            if node.right:
                queue.append(node.right)
        result.append(current_level)
    return result
</CODE>""",
        # 8. Trie (Prefix Tree)
        """<CODE>
class TrieNode:
    def __init__(self) -> None:
        self.children: dict[str, TrieNode] = {}
        self.is_terminal: bool = False

class Trie:
    def __init__(self) -> None:
        self.root = TrieNode()

    def insert(self, word: str) -> None:
        node = self.root
        for ch in word:
            if ch not in node.children:
                node.children[ch] = TrieNode()
            node = node.children[ch]
        node.is_terminal = True

    def starts_with(self, prefix: str) -> bool:
        node = self.root
        for ch in prefix:
            if ch not in node.children:
                return False
            node = node.children[ch]
        return True
</CODE>""",
        # 9. Lowest Common Ancestor (LCA)
        """<CODE>
def lowest_common_ancestor(root: TreeNode | None, p: TreeNode, q: TreeNode) -> TreeNode | None:
    \"\"\"Find LCA of two nodes in a binary tree.\"\"\"
    if not root or root == p or root == q:
        return root
    left = lowest_common_ancestor(root.left, p, q)
    right = lowest_common_ancestor(root.right, p, q)
    if left and right:
        return root
    return left if left is not None else right
</CODE>""",
        # 10. Topological Sort
        """<CODE>
from collections import defaultdict, deque

def topological_sort(num_nodes: int, edges: list[tuple[int, int]]) -> list[int]:
    \"\"\"Kahn's algorithm for topological sorting of DAG.\"\"\"
    in_degree = [0] * num_nodes
    adj = defaultdict(list)
    for u, v in edges:
        adj[u].append(v)
        in_degree[v] += 1
    queue = deque([i for i in range(num_nodes) if in_degree[i] == 0])
    order = []
    while queue:
        u = queue.popleft()
        order.append(u)
        for v in adj[u]:
            in_degree[v] -= 1
            if in_degree[v] == 0:
                queue.append(v)
    if len(order) != num_nodes:
        raise ValueError("Graph contains cycles")
    return order
</CODE>""",
        # 11. FastAPI CRUD REST Handler
        """<CODE>
from fastapi import FastAPI, HTTPException, status
from pydantic import BaseModel, Field

app = FastAPI(title="Projects API")

class ProjectCreate(BaseModel):
    name: str = Field(min_length=1, max_length=100)
    description: str = Field(default="", max_length=500)

class ProjectResponse(ProjectCreate):
    id: int
    active: bool = True

_DB: dict[int, ProjectResponse] = {}

@app.post("/projects", response_model=ProjectResponse, status_code=status.HTTP_201_CREATED)
async def create_project(payload: ProjectCreate) -> ProjectResponse:
    project_id = len(_DB) + 1
    item = ProjectResponse(id=project_id, **payload.model_dump())
    _DB[project_id] = item
    return item

@app.get("/projects/{project_id}", response_model=ProjectResponse)
async def get_project(project_id: int) -> ProjectResponse:
    if project_id not in _DB:
        raise HTTPException(status_code=404, detail="Project not found")
    return _DB[project_id]
</CODE>""",
        # 12. JWT Auth Middleware
        """<CODE>
from typing import Optional
from fastapi import Header, HTTPException, status

async def verify_auth_token(authorization: Optional[str] = Header(None)) -> dict:
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Missing or malformed Authorization header",
            headers={"WWW-Authenticate": "Bearer"},
        )
    token = authorization.split(" ")[1]
    if len(token) < 16:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Invalid authentication token credentials",
        )
    return {"sub": "authenticated_user", "token": token}
</CODE>""",
        # 13. Cursor-based REST Pagination
        """<CODE>
from typing import Generic, Sequence, TypeVar
from pydantic import BaseModel

T = TypeVar("T")

class CursorPage(BaseModel, Generic[T]):
    items: Sequence[T]
    next_cursor: str | None = None
    has_more: bool = False

def paginate_by_cursor(items: list[dict], cursor: str | None, limit: int = 20) -> CursorPage[dict]:
    start_idx = 0
    if cursor is not None:
        for idx, item in enumerate(items):
            if str(item["id"]) == cursor:
                start_idx = idx + 1
                break
    paged = items[start_idx : start_idx + limit]
    has_more = (start_idx + limit) < len(items)
    next_cur = str(paged[-1]["id"]) if (has_more and paged) else None
    return CursorPage(items=paged, next_cursor=next_cur, has_more=has_more)
</CODE>""",
        # 14. Rust Axum REST Endpoint
        """<CODE>
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StatusPayload {
    pub service: String,
    pub operational: bool,
}

pub async fn check_service_health(Path(name): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    if name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let body = StatusPayload {
        service: name,
        operational: true,
    };
    Ok((StatusCode::OK, Json(body)))
}
</CODE>""",
        # 15. Asyncio Worker Pool
        """<CODE>
import asyncio
from typing import Any, Callable, Coroutine

class AsyncWorkerPool:
    \"\"\"Concurrent worker pool with bounded concurrency using asyncio Semaphore.\"\"\"
    def __init__(self, max_concurrency: int = 4) -> None:
        self.semaphore = asyncio.Semaphore(max_concurrency)

    async def execute(self, task_fn: Callable[..., Coroutine[Any, Any, Any]], *args: Any) -> Any:
        async with self.semaphore:
            return await task_fn(*args)

    async def map(self, task_fn: Callable[[Any], Coroutine[Any, Any, Any]], items: list[Any]) -> list[Any]:
        tasks = [self.execute(task_fn, item) for item in items]
        return await asyncio.gather(*tasks)
</CODE>""",
        # 16. Async Retry with Exponential Backoff
        """<CODE>
import asyncio
import random
from typing import Any, Callable, Coroutine

async def retry_with_backoff(
    coro_fn: Callable[..., Coroutine[Any, Any, Any]],
    retries: int = 3,
    base_delay: float = 0.5,
    max_delay: float = 5.0,
    jitter: bool = True,
) -> Any:
    for attempt in range(retries):
        try:
            return await coro_fn()
        except Exception as exc:
            if attempt == retries - 1:
                raise exc
            delay = min(base_delay * (2 ** attempt), max_delay)
            if jitter:
                delay += random.uniform(0.0, 0.2 * delay)
            await asyncio.sleep(delay)
</CODE>""",
        # 17. Async Token Bucket Rate Limiter
        """<CODE>
import asyncio
import time

class TokenBucketLimiter:
    \"\"\"Token bucket algorithm for rate-limiting async calls.\"\"\"
    def __init__(self, rate: float, capacity: float) -> None:
        self.rate = rate
        self.capacity = capacity
        self.tokens = capacity
        self.last_update = time.monotonic()
        self.lock = asyncio.Lock()

    async def acquire(self, tokens: float = 1.0) -> None:
        async with self.lock:
            while True:
                now = time.monotonic()
                elapsed = now - self.last_update
                self.tokens = min(self.capacity, self.tokens + elapsed * self.rate)
                self.last_update = now
                if self.tokens >= tokens:
                    self.tokens -= tokens
                    return
                sleep_duration = (tokens - self.tokens) / self.rate
                await asyncio.sleep(sleep_duration)
</CODE>""",
        # 18. Async Event Bus
        """<CODE>
import asyncio
from typing import Any, Callable, Coroutine

class AsyncEventBus:
    \"\"\"In-memory pub-sub event dispatcher.\"\"\"
    def __init__(self) -> None:
        self._handlers: dict[str, list[Callable[[Any], Coroutine[Any, Any, None]]]] = {}

    def subscribe(self, topic: str, handler: Callable[[Any], Coroutine[Any, Any, None]]) -> None:
        self._handlers.setdefault(topic, []).append(handler)

    async def publish(self, topic: str, event_data: Any) -> None:
        handlers = self._handlers.get(topic, [])
        if handlers:
            await asyncio.gather(*(h(event_data) for h in handlers))
</CODE>""",
        # 19. Async Batch Aggregator
        """<CODE>
import asyncio
from typing import Any, List

class BatchAggregator:
    def __init__(self, batch_size: int = 10, max_wait_sec: float = 0.1) -> None:
        self.batch_size = batch_size
        self.max_wait_sec = max_wait_sec
        self.queue: asyncio.Queue = asyncio.Queue()

    async def add(self, item: Any) -> None:
        await self.queue.put(item)

    async def collect_batch(self) -> List[Any]:
        batch: List[Any] = []
        try:
            item = await asyncio.wait_for(self.queue.get(), timeout=self.max_wait_sec)
            batch.append(item)
            while len(batch) < self.batch_size and not self.queue.empty():
                batch.append(self.queue.get_nowait())
        except asyncio.TimeoutError:
            pass
        return batch
</CODE>""",
        # 20. LRU Cache
        """<CODE>
from typing import Any

class LRUNode:
    def __init__(self, key: str, val: Any) -> None:
        self.key = key
        self.val = val
        self.prev: LRUNode | None = None
        self.next: LRUNode | None = None

class LRUCache:
    def __init__(self, capacity: int) -> None:
        self.capacity = capacity
        self.map: dict[str, LRUNode] = {}
        self.head = LRUNode("", None)
        self.tail = LRUNode("", None)
        self.head.next = self.tail
        self.tail.prev = self.head

    def get(self, key: str) -> Any:
        if key not in self.map:
            return None
        node = self.map[key]
        self._remove(node)
        self._insert_head(node)
        return node.val

    def _remove(self, node: LRUNode) -> None:
        node.prev.next = node.next
        node.next.prev = node.prev

    def _insert_head(self, node: LRUNode) -> None:
        node.next = self.head.next
        node.prev = self.head
        self.head.next.prev = node
        self.head.next = node
</CODE>""",
    ]
    return corpus


def generate_seed_design_corpus() -> List[str]:
    """Generate a rich collection of seed design specs, tokens, and WCAG rules.

    Includes:
    - UX flow specs (Auth, Onboarding, Checkout, Dashboard, Settings, Search)
    - Design token definitions (Color scales, Dark mode semantic, Typography, Spacing, Shadows, Motion)
    - WCAG 2.2 accessibility rules (Contrast, Keyboard Focus, ARIA landmarks, Touch targets, Forms)
    """
    corpus: List[str] = [
        # 1. User Authentication UX Flow
        """<FLOW>
Flow: UserAuthentication
Entrypoint: /auth/login
States:
  - state: IDLE
    trigger: onUserView
    view: LoginForm(email_input, password_input, remember_checkbox, submit_btn)
  - state: SUBMITTING
    trigger: onSubmit(email, password)
    action: disableInputs, showSpinner(submit_btn)
  - state: MFA_CHALLENGE
    trigger: onAuthRequiresMFA
    view: MFACodeInput(digits=6, resend_timer=30s)
  - state: AUTH_SUCCESS
    trigger: onTokenValidated
    action: persistSession, redirect(/dashboard)
  - state: AUTH_ERROR
    trigger: onInvalidCredentials
    action: announceError(aria_live="polite"), highlightField(password_input)
</FLOW>""",
        # 2. Onboarding Wizard Flow
        """<FLOW>
Flow: OnboardingWizard
Steps:
  1. WelcomeScreen: greeting, workspace_name_input, continue_btn
  2. RoleSelection: multi_select_chips[Developer, Designer, PM, Executive]
  3. PreferenceSetup: theme_picker[Light, Dark, System], notification_toggle
  4. Confirmation: summary_card, launch_btn
Handoff:
  - transitions: linear_step_progress
  - allow_back_navigation: true
  - auto_save_state: localStorage
</FLOW>""",
        # 3. Checkout Stepper Flow
        """<FLOW>
Flow: ECommerceCheckout
Steps:
  - Step 1: CartReview(items_table, coupon_input, subtotal_summary)
  - Step 2: ShippingAddress(address_autocomplete, postal_validation, delivery_options)
  - Step 3: PaymentMethod(credit_card_inputs, digital_wallet_buttons, security_badge)
  - Step 4: OrderReviewAndPlace(total_breakdown, place_order_button)
State Guards:
  - preventNextStepIf: shipping_address_invalid OR payment_token_missing
</FLOW>""",
        # 4. Dashboard Workspace Flow
        """<FLOW>
Flow: DashboardWorkspace
States:
  - EMPTY: EmptyStateIllustration(message="No active projects yet", action="Create New Project")
  - LOADING: SkeletonCardGrid(count=6, shimmer=true)
  - POPULATED: ProjectCardGrid(filter_bar, sort_dropdown, pagination_controls)
  - ERROR: ErrorBanner(status="Network Error", retry_action=refreshFeed)
</FLOW>""",
        # 5. Settings & Profile Flow
        """<FLOW>
Flow: UserAccountSettings
Sections:
  - ProfileInfo: avatar_upload(crop_aspect="1:1"), display_name, email_badge
  - Security: password_change_drawer, active_sessions_list(revoke_button)
  - Preferences: language_select, density_toggle[Compact, Comfortable]
Handoff:
  - onUnsavedChanges: showModal(DiscardPrompt)
  - onSuccess: showToast(message="Settings saved successfully", duration=4000ms)
</FLOW>""",
        # 6. Global Search & Autocomplete
        """<FLOW>
Flow: OmnibarSearch
Triggers: [Cmd+K, Ctrl+K]
States:
  - INACTIVE: Closed
  - ACTIVE_EMPTY: RecentSearchesList, QuickActionsList
  - SEARCHING: DebouncedInput(delay=150ms), InlineLoadingIndicator
  - RESULTS: CategoryGroupedList[Files, Symbols, Commands, People]
  - NO_RESULTS: EmptyIllustration(text="No matching commands or files found")
</FLOW>""",
        # 7. Color System Tokens
        """<TOKENS>
Tokens: ColorSystem
palette:
  primary:
    50: "#eef2ff"
    100: "#e0e7ff"
    500: "#6366f1"
    600: "#4f46e5"
    900: "#312e81"
  neutral:
    50: "#f8fafc"
    100: "#f1f5f9"
    200: "#e2e8f0"
    500: "#64748b"
    800: "#1e293b"
    900: "#0f172a"
  semantic:
    success: "#10b981"
    warning: "#f59e0b"
    danger: "#ef4444"
    info: "#3b82f6"
</TOKENS>""",
        # 8. Dark Mode Semantic Tokens
        """<TOKENS>
Tokens: DarkModeSemantic
theme: dark
surfaces:
  background-base: "#0b0f19"
  surface-card: "#111827"
  surface-overlay: "#1f2937"
  surface-hover: "#374151"
borders:
  subtle: "#1f2937"
  strong: "#374151"
  focus-ring: "#818cf8"
text:
  primary: "#f9fafb"
  secondary: "#9ca3af"
  muted: "#6b7280"
  inverse: "#111827"
</TOKENS>""",
        # 9. Typography Tokens
        """<TOKENS>
Tokens: Typography
fontFamilies:
  sans: "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
  mono: "'JetBrains Mono', 'Fira Code', Menlo, Monaco, Consolas, monospace"
fontSizes:
  xs: { size: "12px", lineHeight: "16px", tracking: "0.01em" }
  sm: { size: "14px", lineHeight: "20px", tracking: "0em" }
  base: { size: "16px", lineHeight: "24px", tracking: "-0.01em" }
  lg: { size: "18px", lineHeight: "28px", tracking: "-0.01em" }
  xl: { size: "20px", lineHeight: "28px", tracking: "-0.02em" }
  2xl: { size: "24px", lineHeight: "32px", tracking: "-0.02em" }
</TOKENS>""",
        # 10. Spacing & Grid System Tokens
        """<TOKENS>
Tokens: SpacingGrid
grid-base: 4px
scale:
  space-1: "4px"
  space-2: "8px"
  space-3: "12px"
  space-4: "16px"
  space-6: "24px"
  space-8: "32px"
  space-12: "48px"
  space-16: "64px"
layout:
  container-sm: "640px"
  container-md: "768px"
  container-lg: "1024px"
  container-xl: "1280px"
</TOKENS>""",
        # 11. Elevation & Shadow Tokens
        """<TOKENS>
Tokens: ElevationShadows
elevation:
  none: "none"
  sm: "0 1px 2px 0 rgba(0, 0, 0, 0.05)"
  md: "0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)"
  lg: "0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)"
  xl: "0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)"
radius:
  sm: "4px"
  md: "8px"
  lg: "12px"
  full: "9999px"
</TOKENS>""",
        # 12. Motion & Transitions Tokens
        """<TOKENS>
Tokens: MotionCurves
duration:
  instant: "0ms"
  fast: "150ms"
  normal: "250ms"
  slow: "400ms"
easing:
  standard: "cubic-bezier(0.2, 0.0, 0, 1.0)"
  entrance: "cubic-bezier(0.0, 0.0, 0.2, 1.0)"
  exit: "cubic-bezier(0.4, 0.0, 1.0, 1.0)"
</TOKENS>""",
        # 13. WCAG 2.2 AA Contrast Guidelines
        """<CRITIQUE>
Guideline: WCAG 2.2 AA Color Contrast (Criterion 1.4.3 & 1.4.11)
Specifications:
  - Normal Text (< 18pt or < 14pt bold): minimum contrast ratio 4.5:1 against background.
  - Large Text (>= 18pt or >= 14pt bold): minimum contrast ratio 3.0:1 against background.
  - UI Components & Graphical Objects: minimum contrast ratio 3.0:1 against adjacent colors.
  - Interactive States: active, focused, and hovered buttons must maintain distinct 3:1 luminance contrast.
Checklist:
  [ ] Text readability on primary buttons verified with WebAIM or APCA contrast calculator.
  [ ] Subdued helper text and placeholders meet 4.5:1 threshold.
  [ ] Disabled elements exempted, but recommended to stay above 2.5:1 for clarity.
</CRITIQUE>""",
        # 14. WCAG Keyboard Focus Guidelines
        """<CRITIQUE>
Guideline: WCAG 2.2 AA Keyboard Navigation & Focus Indicator (Criterion 2.4.7 & 2.4.11)
Specifications:
  - All interactive elements must be focusable via Tab and Shift+Tab in logical sequence.
  - Focus Ring: minimum 2px solid indicator with high contrast (>= 3:1) against background.
  - No keyboard trap: modals must trap Tab focus inside modal, and Escape key must dismiss.
  - Skip Links: provide visible 'Skip to main content' link as the first focusable element.
Implementation:
  CSS: :focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
</CRITIQUE>""",
        # 15. WCAG ARIA Landmark Guidelines
        """<CRITIQUE>
Guideline: WCAG 2.2 AA ARIA Landmarks & Screen Reader Compatibility (Criterion 1.3.1 & 4.1.2)
Specifications:
  - Every page must contain landmarks: <header role="banner">, <nav role="navigation">, <main role="main">.
  - Live Regions: status alerts and async updates require aria-live="polite" or role="status".
  - Dialogs: require role="dialog", aria-modal="true", and aria-labelledby referencing title element.
  - Expandable Widgets: accordions and dropdowns require aria-expanded="true|false" and aria-controls.
</CRITIQUE>""",
        # 16. WCAG Touch Target Sizing Guidelines
        """<CRITIQUE>
Guideline: WCAG 2.2 AA Target Size Minimum (Criterion 2.5.8)
Specifications:
  - Pointer target area must be at least 24x24 CSS pixels, preferably 44x44 CSS pixels on touch devices.
  - Sufficient spacing: targets smaller than 44x44px must maintain at least 8px separation between edges.
  - Responsive reflow: layout must scale to 200% zoom without horizontal scrolling or clipping.
</CRITIQUE>""",
        # 17. WCAG Accessible Forms & Errors
        """<CRITIQUE>
Guideline: WCAG 2.2 AA Form Validation and Accessible Errors (Criterion 3.3.1 & 3.3.2)
Specifications:
  - Every input must have programmatic label association: <label for="field-id"> or aria-label.
  - Error messages must link to input using aria-describedby="field-error-id".
  - Invalid state flagged via aria-invalid="true".
  - Grouped controls (radio buttons, checkboxes) must use <fieldset> with <legend>.
</CRITIQUE>""",
        # 18. WCAG Motion & Sensory Adaptation
        """<CRITIQUE>
Guideline: WCAG 2.2 AA Animation & Reduced Motion (Criterion 2.3.3)
Specifications:
  - Respect user preference via @media (prefers-reduced-motion: reduce).
  - Animations longer than 5 seconds must provide pause, stop, or hide controls.
  - Content must not flash more than 3 times in any 1-second period (seizure safety).
</CRITIQUE>""",
        # 19. Component Token Spec: Button System
        """<TOKENS>
Component: ButtonSystem
variants:
  primary:
    bg: "var(--color-primary-600)"
    color: "#ffffff"
    hover-bg: "var(--color-primary-700)"
    focus-ring: "var(--color-primary-300)"
  secondary:
    bg: "var(--color-neutral-100)"
    color: "var(--color-neutral-900)"
    border: "1px solid var(--color-neutral-300)"
  destructive:
    bg: "var(--color-semantic-danger)"
    color: "#ffffff"
sizes:
  sm: { height: "32px", padding: "0 12px", font-size: "14px", radius: "6px" }
  md: { height: "40px", padding: "0 16px", font-size: "16px", radius: "8px" }
  lg: { height: "48px", padding: "0 24px", font-size: "18px", radius: "10px" }
</TOKENS>""",
        # 20. Component Token Spec: Modal System
        """<TOKENS>
Component: ModalDialogSystem
specs:
  overlay:
    backdrop-filter: "blur(4px)"
    background: "rgba(15, 23, 42, 0.6)"
  dialog:
    max-width: "560px"
    border-radius: "16px"
    padding: "24px"
    background: "var(--surface-overlay)"
    box-shadow: "var(--shadow-xl)"
    border: "1px solid var(--border-subtle)"
  header:
    font-size: "20px"
    font-weight: 600
    margin-bottom: "16px"
</TOKENS>""",
    ]
    return corpus


def save_corpus_to_file(corpus: List[str], file_path: str) -> None:
    """Save a list of corpus snippets joined by double newline to file."""
    os.makedirs(os.path.dirname(os.path.abspath(file_path)), exist_ok=True)
    with open(file_path, "w", encoding="utf-8") as f:
        f.write("\n\n".join(corpus))
