// Chain of Responsibility: Expense Approval Workflow
//
// Demonstrates the classic linked-list chain pattern. Different approval
// levels handle different expense amounts: team lead ($100), manager ($1000),
// director ($10000), VP (unlimited). Requests escalate up the chain until
// someone can approve.
//
// Run: go run ./approval/
package main

import (
	"fmt"
	"math"
	"time"
)

// --- Domain Types ---

// ExpenseRequest represents a request for expense approval
type ExpenseRequest struct {
	ID          string
	Employee    string
	Amount      float64
	Category    string
	Description string
	SubmittedAt time.Time
}

// ApprovalResult holds the outcome of the approval chain
type ApprovalResult struct {
	Approved bool
	Approver string
	Level    string
	Notes    string
}

func (r ApprovalResult) String() string {
	if r.Approved {
		return fmt.Sprintf("APPROVED by %s (%s) -- %s", r.Approver, r.Level, r.Notes)
	}
	return fmt.Sprintf("REJECTED by %s (%s) -- %s", r.Approver, r.Level, r.Notes)
}

// --- Chain of Responsibility (Linked-List Style) ---

// Approver defines the handler interface
type Approver interface {
	SetNext(approver Approver) Approver
	Approve(req ExpenseRequest) ApprovalResult
}

// BaseApprover provides the default chain traversal behavior.
// Concrete approvers embed this to get SetNext and fallthrough behavior.
type BaseApprover struct {
	next Approver
}

func (b *BaseApprover) SetNext(approver Approver) Approver {
	b.next = approver
	return approver // return the argument for fluent chaining
}

func (b *BaseApprover) Approve(req ExpenseRequest) ApprovalResult {
	if b.next != nil {
		return b.next.Approve(req)
	}
	// End of chain -- no one could approve
	return ApprovalResult{
		Approved: false,
		Approver: "system",
		Level:    "chain-end",
		Notes:    "no approver in the chain could handle this request",
	}
}

// --- Concrete Approvers ---

// TeamLead can approve expenses up to their limit.
// They also reject certain categories outright.
type TeamLead struct {
	BaseApprover
	Name  string
	Limit float64
}

func (t *TeamLead) Approve(req ExpenseRequest) ApprovalResult {
	fmt.Printf("  [%s (Team Lead)] reviewing $%.2f expense...\n", t.Name, req.Amount)

	// Team leads reject entertainment expenses -- company policy
	if req.Category == "entertainment" {
		return ApprovalResult{
			Approved: false,
			Approver: t.Name,
			Level:    "team-lead",
			Notes:    "entertainment expenses require manager approval regardless of amount",
		}
	}

	if req.Amount <= t.Limit {
		return ApprovalResult{
			Approved: true,
			Approver: t.Name,
			Level:    "team-lead",
			Notes:    fmt.Sprintf("within team lead limit ($%.2f)", t.Limit),
		}
	}

	fmt.Printf("  [%s (Team Lead)] $%.2f exceeds limit ($%.2f), escalating...\n",
		t.Name, req.Amount, t.Limit)
	return t.BaseApprover.Approve(req)
}

// Manager can approve larger expenses and entertainment category.
type Manager struct {
	BaseApprover
	Name  string
	Limit float64
}

func (m *Manager) Approve(req ExpenseRequest) ApprovalResult {
	fmt.Printf("  [%s (Manager)] reviewing $%.2f expense...\n", m.Name, req.Amount)

	if req.Amount <= m.Limit {
		return ApprovalResult{
			Approved: true,
			Approver: m.Name,
			Level:    "manager",
			Notes:    fmt.Sprintf("within manager limit ($%.2f)", m.Limit),
		}
	}

	fmt.Printf("  [%s (Manager)] $%.2f exceeds limit ($%.2f), escalating...\n",
		m.Name, req.Amount, m.Limit)
	return m.BaseApprover.Approve(req)
}

// Director handles large expenses and can override category restrictions.
type Director struct {
	BaseApprover
	Name  string
	Limit float64
}

func (d *Director) Approve(req ExpenseRequest) ApprovalResult {
	fmt.Printf("  [%s (Director)] reviewing $%.2f expense...\n", d.Name, req.Amount)

	if req.Amount <= d.Limit {
		return ApprovalResult{
			Approved: true,
			Approver: d.Name,
			Level:    "director",
			Notes:    fmt.Sprintf("within director limit ($%.2f)", d.Limit),
		}
	}

	fmt.Printf("  [%s (Director)] $%.2f exceeds limit ($%.2f), escalating...\n",
		d.Name, req.Amount, d.Limit)
	return d.BaseApprover.Approve(req)
}

// VP is the final approver with an unlimited budget.
// If it gets here and the VP rejects it, the request is denied.
type VP struct {
	BaseApprover
	Name string
}

func (v *VP) Approve(req ExpenseRequest) ApprovalResult {
	fmt.Printf("  [%s (VP)] reviewing $%.2f expense...\n", v.Name, req.Amount)

	// VP can approve anything, but rejects suspiciously large amounts
	if req.Amount > 100000 {
		return ApprovalResult{
			Approved: false,
			Approver: v.Name,
			Level:    "vp",
			Notes:    "expenses over $100,000 require board approval",
		}
	}

	return ApprovalResult{
		Approved: true,
		Approver: v.Name,
		Level:    "vp",
		Notes:    "VP approval -- no limit",
	}
}

// --- Main ---

func main() {
	fmt.Println("=== Chain of Responsibility: Expense Approval ===\n")

	// Build the approval chain: TeamLead -> Manager -> Director -> VP
	teamLead := &TeamLead{Name: "Alice", Limit: 100}
	manager := &Manager{Name: "Bob", Limit: 1000}
	director := &Director{Name: "Carol", Limit: 10000}
	vp := &VP{Name: "Dave"}

	// Fluent chain construction
	teamLead.SetNext(manager).SetNext(director).SetNext(vp)

	// Test cases
	requests := []ExpenseRequest{
		{
			ID:          "EXP-001",
			Employee:    "Junior Dev",
			Amount:      45.00,
			Category:    "office-supplies",
			Description: "Keyboard and mouse",
		},
		{
			ID:          "EXP-002",
			Employee:    "Senior Dev",
			Amount:      750.00,
			Category:    "conference",
			Description: "GopherCon ticket",
		},
		{
			ID:          "EXP-003",
			Employee:    "Tech Lead",
			Amount:      5000.00,
			Category:    "equipment",
			Description: "New server for staging environment",
		},
		{
			ID:          "EXP-004",
			Employee:    "CTO",
			Amount:      25000.00,
			Category:    "infrastructure",
			Description: "Annual cloud hosting contract",
		},
		{
			ID:          "EXP-005",
			Employee:    "Marketing Lead",
			Amount:      50.00,
			Category:    "entertainment",
			Description: "Team dinner",
		},
		{
			ID:          "EXP-006",
			Employee:    "CEO",
			Amount:      500000.00,
			Category:    "acquisition",
			Description: "Purchase competitor's IP",
		},
	}

	for _, req := range requests {
		fmt.Printf("Request %s: $%.2f for %q (%s)\n", req.ID, req.Amount, req.Description, req.Category)
		result := teamLead.Approve(req)
		fmt.Printf("  Result: %s\n\n", result)
	}

	// Demonstrate chain statistics
	fmt.Println("--- Approval Limits ---")
	fmt.Printf("Team Lead: $%.2f\n", teamLead.Limit)
	fmt.Printf("Manager:   $%.2f\n", manager.Limit)
	fmt.Printf("Director:  $%.2f\n", director.Limit)
	fmt.Printf("VP:        $%.2f (unlimited)\n", math.MaxFloat64)
}
