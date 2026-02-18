// Run with: go run ./formats/
//
// Demonstrates: encoding/xml basics, encoding/csv read/write,
// and a multi-format serializer that produces the same data in JSON, CSV, and XML.
package main

import (
	"bytes"
	"encoding/csv"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"io"
	"log"
	"strconv"
	"strings"
	"time"
)

// ============================================================================
// Shared domain type
//
// Order is used across all three formats.
// The xml and json tags control serialization per-format.
// CSV has no tag system — field order is managed manually.
// ============================================================================

type Order struct {
	OrderID    string    `json:"order_id"    xml:"order_id"`
	CustomerID string    `json:"customer_id" xml:"customer_id"`
	Status     string    `json:"status"      xml:"status"`
	AmountCents int64    `json:"amount_cents" xml:"amount_cents"`
	CreatedAt  time.Time `json:"created_at"  xml:"created_at"`
}

// ============================================================================
// encoding/xml
// ============================================================================

// OrderList is the XML root element wrapping a list of orders.
// XMLName sets the element name; items serialize as repeated <order> elements.
type OrderList struct {
	XMLName xml.Name `xml:"orders"`
	Items   []Order  `xml:"order"`
}

func xmlDemo() {
	fmt.Println("=== encoding/xml ===")

	now := time.Date(2026, 2, 18, 10, 0, 0, 0, time.UTC)
	list := OrderList{
		Items: []Order{
			{OrderID: "ord_001", CustomerID: "cust_A", Status: "shipped", AmountCents: 9999, CreatedAt: now},
			{OrderID: "ord_002", CustomerID: "cust_B", Status: "pending", AmountCents: 4500, CreatedAt: now},
		},
	}

	// Marshal to XML
	data, err := xml.MarshalIndent(list, "", "  ")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println("XML output:")
	fmt.Println(string(data))

	// Unmarshal back
	var decoded OrderList
	if err := xml.Unmarshal(data, &decoded); err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Decoded %d orders\n", len(decoded.Items))
	fmt.Printf("First: %s — %s\n", decoded.Items[0].OrderID, decoded.Items[0].Status)
}

// ============================================================================
// encoding/csv
//
// encoding/csv works with []string rows — no struct tags.
// Column order is controlled manually at read and write time.
// Always call w.Flush() and check w.Error() after writing.
// ============================================================================

var csvHeaders = []string{"order_id", "customer_id", "status", "amount_cents", "created_at"}

func orderToRecord(o Order) []string {
	return []string{
		o.OrderID,
		o.CustomerID,
		o.Status,
		strconv.FormatInt(o.AmountCents, 10),
		o.CreatedAt.UTC().Format(time.RFC3339),
	}
}

func recordToOrder(record []string) (Order, error) {
	if len(record) != 5 {
		return Order{}, fmt.Errorf("expected 5 fields, got %d", len(record))
	}
	amount, err := strconv.ParseInt(record[3], 10, 64)
	if err != nil {
		return Order{}, fmt.Errorf("invalid amount_cents %q: %w", record[3], err)
	}
	createdAt, err := time.Parse(time.RFC3339, record[4])
	if err != nil {
		return Order{}, fmt.Errorf("invalid created_at %q: %w", record[4], err)
	}
	return Order{
		OrderID:     record[0],
		CustomerID:  record[1],
		Status:      record[2],
		AmountCents: amount,
		CreatedAt:   createdAt,
	}, nil
}

func csvDemo() {
	fmt.Println("\n=== encoding/csv ===")

	now := time.Date(2026, 2, 18, 10, 0, 0, 0, time.UTC)
	orders := []Order{
		{OrderID: "ord_001", CustomerID: "cust_A", Status: "shipped", AmountCents: 9999, CreatedAt: now},
		{OrderID: "ord_002", CustomerID: "cust_B", Status: "pending", AmountCents: 4500, CreatedAt: now},
		{OrderID: "ord_003", CustomerID: "cust_C", Status: "refunded", AmountCents: 1200, CreatedAt: now},
	}

	// Write CSV to a buffer
	var buf bytes.Buffer
	w := csv.NewWriter(&buf)

	w.Write(csvHeaders)
	for _, o := range orders {
		w.Write(orderToRecord(o))
	}
	w.Flush() // must flush before reading the buffer
	if err := w.Error(); err != nil {
		log.Fatal(err)
	}

	fmt.Println("CSV output:")
	fmt.Println(buf.String())

	// Read it back
	r := csv.NewReader(strings.NewReader(buf.String()))
	_, _ = r.Read() // skip header row

	var decoded []Order
	for {
		record, err := r.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Fatal(err)
		}
		o, err := recordToOrder(record)
		if err != nil {
			log.Fatal(err)
		}
		decoded = append(decoded, o)
	}

	fmt.Printf("Decoded %d orders from CSV\n", len(decoded))
	for _, o := range decoded {
		fmt.Printf("  %s: %s — %d cents\n", o.OrderID, o.Status, o.AmountCents)
	}
}

// ============================================================================
// Multi-format serializer
//
// A production pattern: a type that can render itself in multiple formats
// based on a content-type negotiation or format parameter.
// The domain type (Order) doesn't change — only the serialization layer does.
// ============================================================================

type Format string

const (
	FormatJSON Format = "json"
	FormatCSV  Format = "csv"
	FormatXML  Format = "xml"
)

// Serialize writes a slice of orders to w in the specified format.
func Serialize(orders []Order, format Format, w io.Writer) error {
	switch format {
	case FormatJSON:
		enc := json.NewEncoder(w)
		enc.SetIndent("", "  ")
		return enc.Encode(orders)

	case FormatCSV:
		cw := csv.NewWriter(w)
		if err := cw.Write(csvHeaders); err != nil {
			return err
		}
		for _, o := range orders {
			if err := cw.Write(orderToRecord(o)); err != nil {
				return err
			}
		}
		cw.Flush()
		return cw.Error()

	case FormatXML:
		list := OrderList{Items: orders}
		data, err := xml.MarshalIndent(list, "", "  ")
		if err != nil {
			return err
		}
		_, err = w.Write(data)
		return err

	default:
		return fmt.Errorf("unsupported format %q", format)
	}
}

func multiFormatDemo() {
	fmt.Println("\n=== Multi-format serializer ===")

	now := time.Date(2026, 2, 18, 10, 0, 0, 0, time.UTC)
	orders := []Order{
		{OrderID: "ord_001", CustomerID: "cust_A", Status: "shipped", AmountCents: 9999, CreatedAt: now},
	}

	for _, format := range []Format{FormatJSON, FormatCSV, FormatXML} {
		fmt.Printf("\n-- %s --\n", format)
		var buf bytes.Buffer
		if err := Serialize(orders, format, &buf); err != nil {
			log.Fatal(err)
		}
		fmt.Print(buf.String())
	}
}

// ============================================================================
// CSV configuration options
// ============================================================================

func csvConfigOptions() {
	fmt.Println("\n=== CSV configuration options ===")

	// Tab-separated values (TSV) — same package, different delimiter
	tsv := "order_id\tcustomer_id\tstatus\n" +
		"ord_001\tcust_A\tshipped\n" +
		"ord_002\tcust_B\tpending\n"

	r := csv.NewReader(strings.NewReader(tsv))
	r.Comma = '\t' // tab delimiter

	records, err := r.ReadAll()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("TSV rows: %d\n", len(records))
	for _, rec := range records {
		fmt.Printf("  %v\n", rec)
	}

	// Comment lines — skip lines starting with #
	commented := `# This is a comment
order_id,customer_id,status
ord_001,cust_A,shipped
# Another comment
ord_002,cust_B,pending
`
	r2 := csv.NewReader(strings.NewReader(commented))
	r2.Comment = '#'
	r2.Read() // skip header
	allRecords, _ := r2.ReadAll()
	fmt.Printf("\nCSV with comments — data rows: %d\n", len(allRecords))
}

func main() {
	xmlDemo()
	csvDemo()
	multiFormatDemo()
	csvConfigOptions()
}
