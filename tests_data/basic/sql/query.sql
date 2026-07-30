CREATE TABLE employees (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    department VARCHAR(50),
    salary DECIMAL(10, 2)
);

INSERT INTO employees (id, name, department, salary) VALUES
    (1, 'Alice Johnson', 'Engineering', 95000.00),
    (2, 'Bob Smith', 'Marketing', 72000.00),
    (3, 'Carol White', 'Engineering', 98000.00);

SELECT name, salary
FROM employees
WHERE department = 'Engineering'
ORDER BY salary DESC;
