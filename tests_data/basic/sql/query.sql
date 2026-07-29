CREATE TABLE employees (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    department VARCHAR(50),
    salary DECIMAL(10, 2)
);

INSERT INTO employees (id, name, department, salary) VALUES
    (1, 'Alice', 'Engineering', 95000.00),
    (2, 'Bob', 'Marketing', 72000.00),
    (3, 'Carol', 'Engineering', 105000.00),
    (4, 'Dave', 'HR', 68000.00);

SELECT department, AVG(salary) AS avg_salary
FROM employees
GROUP BY department
ORDER BY avg_salary DESC;
