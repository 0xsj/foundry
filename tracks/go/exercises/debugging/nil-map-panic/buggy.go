package cache

// Cache stores string key-value pairs in memory.
type Cache struct {
	data map[string]string
}

// NewCache creates a new cache instance.
func NewCache() *Cache {
	return &Cache{}
}

// Set stores a value for the given key.
func (c *Cache) Set(key, value string) {
	c.data[key] = value  // BUG: This will panic if data is nil
}

// Get retrieves a value for the given key.
// Returns the value and true if found, or empty string and false if not found.
func (c *Cache) Get(key string) (string, bool) {
	val, ok := c.data[key]
	return val, ok
}

// Delete removes a key from the cache.
func (c *Cache) Delete(key string) {
	delete(c.data, key)
}
